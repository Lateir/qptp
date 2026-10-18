use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{io::{self, Read, Write}, net::{TcpStream}, path::{Path, PathBuf}, process::Command, sync::{atomic::{AtomicBool, AtomicU64, Ordering}, Arc, Mutex}, thread, time::{Duration, Instant}};
use tauri::{menu::{Menu, MenuItem, PredefinedMenuItem}, tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent}, AppHandle, Emitter, Manager, RunEvent, State, WindowEvent, Wry};
mod extra_ipc;
mod discovery;
mod touch;
#[cfg(windows)]
pub mod steamvr_install;

const PORT: u16 = 27182;

const HAPTIC_DURATION_MS: u16 = 2;
const MIN_MODULE_VERSION_CODE: u64 = 9; // QPR3 and QPV1 are required.
const MIN_MODULE_VERSION: &str = "v3.3";
const CONTACT_MODULE_VERSION_CODE: u64 = 12; // QPR4 adds contact size and flag on request.
const SIDES: [&str; 2] = ["left", "right"];

#[derive(Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
struct UserConfig { transport: String, input_mode: String, press_threshold: f32, release_threshold: f32, haptic_amplitude: f32, language: String, steamvr_lifecycle: bool, steamvr_driver_mode: String, fusion_quest3: String, touchpad: touch::Settings }
impl Default for UserConfig {
    fn default()->Self { Self { transport:"lan".into(), input_mode:"touchpad".into(), press_threshold:0.30, release_threshold:0.20, haptic_amplitude:0.20, language:system_language().into(), steamvr_lifecycle:true, steamvr_driver_mode:"classic".into(), fusion_quest3:"ask".into(), touchpad:touch::Settings::default() } }
}
fn valid_mode(mode:&str)->bool {matches!(mode,"touchpad"|"button"|"two_buttons")}
fn valid_driver_mode(mode:&str)->bool {matches!(mode,"classic"|"fusion")}
// Whether controllers reporting a Quest 3 model are Quest Pro hardware: not asked yet, confirmed, or declined.
fn valid_quest3_answer(answer:&str)->bool {matches!(answer,"ask"|"pro"|"ignore")}
fn valid_language(language:&str)->bool {matches!(language,"ru"|"en"|"zh-CN"|"hi"|"es"|"ar"|"fr"|"bn"|"pt-BR"|"id"|"ja"|"de")}
fn match_language(locale:&str)->Option<&'static str>{
    let locale=locale.trim().replace('_',"-").to_ascii_lowercase();
    let primary=locale.split('-').next()?;
    match primary {
        "en"=>Some("en"),"hi"=>Some("hi"),"es"=>Some("es"),"ar"=>Some("ar"),
        "fr"=>Some("fr"),"bn"=>Some("bn"),"ru"=>Some("ru"),"id"|"in"=>Some("id"),
        "de"=>Some("de"),"ja"=>Some("ja"),"pt"=>Some("pt-BR"),
        "zh" if locale=="zh"||locale.starts_with("zh-cn")||locale.starts_with("zh-sg")||locale.starts_with("zh-hans")=>Some("zh-CN"),
        _=>None,
    }
}
fn choose_language<'a>(locales:impl IntoIterator<Item=&'a str>)->&'static str{
    locales.into_iter().find_map(match_language).unwrap_or("en")
}
#[cfg(windows)]
fn system_language()->&'static str{
    use windows_sys::Win32::Globalization::{GetUserPreferredUILanguages,MUI_LANGUAGE_NAME};
    let mut count=0u32;let mut size=0u32;
    if unsafe{GetUserPreferredUILanguages(MUI_LANGUAGE_NAME,&mut count,std::ptr::null_mut(),&mut size)}==0||size==0{return "en"}
    let mut buffer=vec![0u16;size as usize];
    if unsafe{GetUserPreferredUILanguages(MUI_LANGUAGE_NAME,&mut count,buffer.as_mut_ptr(),&mut size)}==0{return "en"}
    let locales:Vec<_>=buffer.split(|&unit|unit==0).take_while(|part|!part.is_empty()).filter_map(|part|String::from_utf16(part).ok()).collect();
    choose_language(locales.iter().map(String::as_str))
}
#[cfg(not(windows))]
fn system_language()->&'static str{
    let locale=std::env::var("LC_ALL").or_else(|_|std::env::var("LC_MESSAGES")).or_else(|_|std::env::var("LANG")).unwrap_or_default();
    choose_language([locale.as_str()])
}
fn valid_threshold(value:f32)->bool {value.is_finite()&&(0.0..=1.0).contains(&value)}

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct Sensor { x: u16, y: u16, force: f32, stylus: f32, trigger_proximity: f32, trigger_slide: f32, size: Option<f32>, touch: Option<bool> }
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    phase: String, message: String, transport: String, endpoint: Option<String>,
    protocol: Option<String>, module_version: Option<Value>, minimum_module_version: &'static str, module_update_required: bool, samples: u64, left: Sensor, right: Sensor, status: Option<Value>,
    input_mode: String, press_threshold: f32, release_threshold: f32, haptic_amplitude: f32, language: String, left_buttons: [bool;2], right_buttons: [bool;2],
    steamvr_connected: bool, steamvr_lifecycle: bool, steamvr_driver_mode: String, fusion_quest3: String, steamvr_driver: extra_ipc::DriverReport,
    touchpad: touch::Settings, left_pad: touch::Output, right_pad: touch::Output, contact_size_available: bool,
    #[serde(skip)] engine: touch::Engine,
}
impl Default for Snapshot {
    fn default() -> Self { Self { phase:"searching".into(), message:"Поиск устройства…".into(), transport:"lan".into(), endpoint:None, protocol:None, module_version:None, minimum_module_version:MIN_MODULE_VERSION, module_update_required:false, samples:0, left:Sensor::default(), right:Sensor::default(), status:None,input_mode:"touchpad".into(),press_threshold:0.30,release_threshold:0.20,haptic_amplitude:0.20,language:"ru".into(),left_buttons:[false;2],right_buttons:[false;2],steamvr_connected:false,steamvr_lifecycle:true,steamvr_driver_mode:"classic".into(),fusion_quest3:"ask".into(),steamvr_driver:extra_ipc::DriverReport::default(),touchpad:touch::Settings::default(),left_pad:touch::Output::default(),right_pad:touch::Output::default(),contact_size_available:false,engine:touch::Engine::default() } }
}
struct StreamState { snapshot: Mutex<Snapshot>, config_lock: Mutex<()>, generation: AtomicU64, adb_used: AtomicBool }
type Shared = Arc<StreamState>;
#[derive(Clone, Copy, PartialEq, Eq)]
struct TrayController { connected: bool, battery: Option<u8> }
#[derive(Clone, Copy, PartialEq, Eq)]
struct TrayStatus { left: TrayController, right: TrayController }
struct Tray { controllers: MenuItem<Wry>, show: MenuItem<Wry>, quit: MenuItem<Wry>, last: Mutex<Option<TrayStatus>> }

fn controller_connected(s:&Snapshot, side:&str)->bool {
    if s.phase!="connected" {return false}
    let Some(status)=&s.status else {return false};
    let value=status.get(side).or_else(||status.get("controllers").and_then(|c|c.get(side)));
    value.and_then(|v|v.get("connected")).and_then(Value::as_bool).unwrap_or(false)
}
fn tray_controller(s:&Snapshot, side:&str)->TrayController {
    let connected=controller_connected(s,side);
    let battery=if connected {
        s.status.as_ref()
            .and_then(|status|status.get(side).or_else(||status.get("controllers").and_then(|c|c.get(side))))
            .and_then(|value|value.get("battery_percent").and_then(Value::as_f64)
                .or_else(||value.get("battery").and_then(Value::as_f64)))
            .filter(|value|value.is_finite())
            .map(|value|value.round().clamp(0.0,100.0) as u8)
    } else {None};
    TrayController{connected,battery}
}
fn tray_status(s:&Snapshot)->TrayStatus {
    TrayStatus{left:tray_controller(s,"left"),right:tray_controller(s,"right")}
}
fn tray_label(status:TrayStatus)->String {
    fn part(side:&str, controller:TrayController)->String {
        let dot=if controller.connected{"✓"}else{"○"};
        let battery=controller.battery.map(|value|format!("{value}%")).unwrap_or_else(||"—".into());
        format!("{dot} {side} {battery}")
    }
    format!("{}    {}",part("L",status.left),part("R",status.right))
}
fn update_tray(app:&AppHandle, status:TrayStatus){
    let Some(tray)=app.try_state::<Tray>() else {return};
    let mut last=tray.last.lock().unwrap();
    if *last==Some(status) {return}
    *last=Some(status);
    let _=tray.controllers.set_text(tray_label(status));
}
fn controller_live(s:&Snapshot, side:&str)->bool {s.phase=="connected"&&(s.status.is_none()||controller_connected(s,side))}
fn refresh_pads(s:&mut Snapshot, now:u64){
    for (index,side) in SIDES.into_iter().enumerate(){
        let enabled=s.input_mode=="touchpad"&&controller_live(s,side);
        let output=s.engine.step(index,now,&s.touchpad,enabled);
        if index==0 {s.left_pad=output} else {s.right_pad=output}
    }
}
// Re-evaluates both pads and sends the result to the SteamVR driver.
fn send_driver(s:&mut Snapshot, now:u64){
    refresh_pads(s,now);
    (s.steamvr_connected,s.steamvr_driver)=extra_ipc::publish(s);
    s.engine.last_sent=now;
}
fn publish(app:&AppHandle, state:&Shared, change:impl FnOnce(&mut Snapshot)) {
    // Tray is updated after the lock is released: menu calls block on the main thread.
    let status=match state.snapshot.lock() {Ok(mut snapshot)=>{change(&mut snapshot);send_driver(&mut snapshot,touch::now_us());let _=app.emit("stream-state",snapshot.clone());tray_status(&snapshot)},Err(_)=>return};
    update_tray(app,status);
}
fn adb_path(app:&AppHandle)->Result<PathBuf,String>{
    let dir=app.path().resource_dir().map_err(|e|e.to_string())?;
    let p=dir.join("resources").join("platform-tools").join("adb.exe");
    if p.exists(){Ok(p)}else{Err(format!("Встроенный ADB не найден: {}",p.display()))}
}
fn adb(path:&PathBuf)->Command{
    let mut cmd=Command::new(path);
    #[cfg(windows)]{use std::os::windows::process::CommandExt;cmd.creation_flags(0x08000000);} // CREATE_NO_WINDOW
    cmd
}
fn adb_forward(app:&AppHandle,state:&Shared)->Result<(PathBuf,bool),String>{
    let path=adb_path(app)?;
    state.adb_used.store(true,Ordering::SeqCst);
    let output=adb(&path).args(["forward","--no-rebind","tcp:27182","tcp:27182"]).output().map_err(|e|format!("Не удалось запустить ADB: {e}"))?;
    if output.status.success(){return Ok((path,true))}
    let serial=adb(&path).arg("get-serialno").output().map_err(|e|e.to_string())?;
    let listing=adb(&path).args(["forward","--list"]).output().map_err(|e|e.to_string())?;
    let id=String::from_utf8_lossy(&serial.stdout).trim().to_owned();
    if serial.status.success() && listing.status.success() && String::from_utf8_lossy(&listing.stdout).lines().any(|line|{
        let p:Vec<_>=line.split_whitespace().collect();p.len()==3&&p[0]==id&&p[1]=="tcp:27182"&&p[2]=="tcp:27182"
    }){return Ok((path,false))}
    Err(format!("Не удалось настроить USB: {}",String::from_utf8_lossy(&output.stderr).trim()))
}
fn read_exact_checked(stream:&mut TcpStream, bytes:&mut [u8], state:&Shared, generation:u64)->io::Result<()> {
    read_exact_deadline(stream,bytes,Duration::from_secs(5),||state.generation.load(Ordering::SeqCst)!=generation)
}
fn read_exact_deadline(stream:&mut impl Read,bytes:&mut [u8],timeout:Duration,cancelled:impl Fn()->bool)->io::Result<()> {
    let deadline=Instant::now()+timeout;
    let mut offset=0;
    while offset<bytes.len() {
        if cancelled() {return Err(io::Error::new(io::ErrorKind::Interrupted,"Отключено"))}
        if Instant::now()>=deadline {return Err(io::Error::new(io::ErrorKind::TimedOut,"Нет данных от модуля в течение 5 секунд"))}
        match stream.read(&mut bytes[offset..]) {
            Ok(0)=>return Err(io::Error::new(io::ErrorKind::UnexpectedEof,"Поток закрыт")),
            Ok(n)=>offset+=n,
            Err(e) if matches!(e.kind(),io::ErrorKind::WouldBlock|io::ErrorKind::TimedOut|io::ErrorKind::Interrupted)=>{},
            Err(e)=>return Err(e),
        }
    }
    Ok(())
}
fn sensor(b:&[u8])->Sensor{
    // Module's Y axis is inverted relative to the pad; flip so 0 is the top.
    Sensor{x:u16::from_le_bytes([b[0],b[1]]),y:255u16.saturating_sub(u16::from_le_bytes([b[2],b[3]])),force:f32_at(b,4),stylus:f32_at(b,8),trigger_proximity:f32_at(b,12),trigger_slide:f32_at(b,16),size:None,touch:None}
}
fn f32_at(b:&[u8],offset:usize)->f32{f32::from_le_bytes(b[offset..offset+4].try_into().unwrap())}
// QPR4 side: x, y, force, size, stylus, proximity, slide, then the contact flag byte and 3 padding bytes.
fn sensor4(b:&[u8])->Sensor{
    Sensor{x:u16::from_le_bytes([b[0],b[1]]),y:255u16.saturating_sub(u16::from_le_bytes([b[2],b[3]])),force:f32_at(b,4),size:Some(f32_at(b,8)),stylus:f32_at(b,12),trigger_proximity:f32_at(b,16),trigger_slide:f32_at(b,20),touch:Some(b[24]&1!=0)}
}
fn touch_raw(sensor:&Sensor)->touch::Raw{touch::Raw{x:sensor.x,y:sensor.y,force:sensor.force,size:sensor.size,touch:sensor.touch}}
fn button_step(previous:[bool;2], sensor:&Sensor, mode:&str, press_threshold:f32, release_threshold:f32)->[bool;2]{
    // Raw (0, 0) is no touch. The decoded Y axis is flipped, so idle is (0, 255).
    if mode=="touchpad" || (sensor.x==0&&sensor.y==255) || !sensor.force.is_finite() || sensor.force<release_threshold {return [false;2]}
    if previous[0]||previous[1] {return previous} // Keep the chosen half until release.
    if sensor.force<press_threshold {return [false;2]}
    if mode=="two_buttons" {[sensor.y<128,sensor.y>=128]} else {[true,false]}
}
fn send_thumb_pulse(stream:&mut TcpStream,side:u8,sequence:u32,amplitude:u8)->io::Result<()> {
    let mut command=[0u8;16];
    command[..4].copy_from_slice(b"QPC1");command[4]=1;command[5]=side;command[6]=2;command[7]=amplitude;
    command[8..10].copy_from_slice(&HAPTIC_DURATION_MS.to_le_bytes());command[10..14].copy_from_slice(&sequence.to_le_bytes());
    stream.write_all(&command)
}
// Asks a module with versionCode >= 12 to stream QPR4 instead of QPR3 for this connection.
fn request_contact_frames(stream:&mut TcpStream,sequence:u32)->io::Result<()> {
    let mut command=[0u8;16];
    command[..4].copy_from_slice(b"QPC1");command[4]=2;command[5]=4;command[10..14].copy_from_slice(&sequence.to_le_bytes());
    stream.write_all(&command)
}
fn new_press(previous:[bool;2],current:[bool;2])->bool {
    !previous.iter().any(|&pressed|pressed) && current.iter().any(|&pressed|pressed)
}
fn haptic_ready(press:bool,pending:Option<u32>,amplitude:u8)->bool{press&&pending.is_none()&&amplitude>0}
fn acknowledge_haptic(pending:&mut [Option<u32>;2],request_id:u32){
    for item in pending {if *item==Some(request_id){*item=None;}}
}
fn read_frame(stream:&mut TcpStream, app:&AppHandle, state:&Shared, generation:u64)->io::Result<()>{
    let mut header=[0u8;4];let mut last_emit=Instant::now()-Duration::from_secs(1);let mut haptic_sequence=0u32;
    let mut pending_haptics:[Option<u32>;2]=[None,None];
    loop {
        if state.generation.load(Ordering::SeqCst)!=generation{return Ok(())}
        read_exact_checked(stream,&mut header,state,generation)?;
        match &header {
            b"QPV1"=>{
                let mut size=[0u8;4];read_exact_checked(stream,&mut size,state,generation)?;
                let size=u32::from_le_bytes(size) as usize;
                if size>4096{return Err(io::Error::new(io::ErrorKind::InvalidData,"Слишком большой кадр версии"))}
                let mut body=vec![0u8;size];read_exact_checked(stream,&mut body,state,generation)?;
                let version:Value=serde_json::from_slice(&body).map_err(io::Error::other)?;
                if state.generation.load(Ordering::SeqCst)!=generation {return Ok(())}
                let code=version.get("versionCode").and_then(Value::as_u64)
                    .ok_or_else(||io::Error::new(io::ErrorKind::InvalidData,"Не указан код версии модуля"))?;
                let required=code<MIN_MODULE_VERSION_CODE;
                publish(app,state,|s|{s.module_version=Some(version);s.module_update_required=required});
                if required {return Err(io::Error::new(io::ErrorKind::InvalidData,"Требуется обновление модуля"))}
                if code>=CONTACT_MODULE_VERSION_CODE {haptic_sequence=haptic_sequence.wrapping_add(1);request_contact_frames(stream,haptic_sequence)?;}
            }
            b"QPR2"=>{
                publish(app,state,|s|s.module_update_required=true);
                return Err(io::Error::new(io::ErrorKind::InvalidData,"Требуется обновление модуля"));
            }
            b"QPA1"=>{
                let mut body=[0u8;12];read_exact_checked(stream,&mut body,state,generation)?;
                let request_id=u32::from_le_bytes(body[..4].try_into().unwrap());
                acknowledge_haptic(&mut pending_haptics,request_id);
            }
            b"QPS1"=>{
                let mut size=[0u8;4];read_exact_checked(stream,&mut size,state,generation)?;
                let size=u32::from_le_bytes(size) as usize;
                if size>4096{return Err(io::Error::new(io::ErrorKind::InvalidData,"Слишком большой статус"))}
                let mut body=vec![0u8;size];read_exact_checked(stream,&mut body,state,generation)?;
                let status:Value=serde_json::from_slice(&body).map_err(io::Error::other)?;
                if state.generation.load(Ordering::SeqCst)!=generation {return Ok(())}
                publish(app,state,|s|{s.status=Some(status);s.phase="connected".into();s.message="Поток сенсоров активен".into();if s.protocol.is_none(){s.protocol=Some("QPR3 / QPS1".into())}if !controller_connected(s,"left"){s.left_buttons=[false;2]}if !controller_connected(s,"right"){s.right_buttons=[false;2]}});
            }
            b"QPR3"|b"QPR4"=>{
                let contact=&header==b"QPR4";
                let mut body=[0u8;68];let body=&mut body[..if contact{68}else{52}];read_exact_checked(stream,body,state,generation)?;
                if state.generation.load(Ordering::SeqCst)!=generation {return Ok(())}
                let (left,right)=if contact{(sensor4(&body[12..40]),sensor4(&body[40..68]))}else{(sensor(&body[12..32]),sensor(&body[32..52]))};
                let mut pulses=[false;2];let mut amplitude=0u8;
                if let Ok(mut s)=state.snapshot.lock(){
                    let previous=(s.left_buttons,s.right_buttons);let now=touch::now_us();
                    s.left=left;s.right=right;s.samples+=1;s.phase="connected".into();s.message="Поток сенсоров активен".into();s.protocol=Some(if contact{"QPR4 / QPS1"}else{"QPR3 / QPS1"}.into());s.contact_size_available=contact;
                    s.left_buttons=if s.status.is_none()||controller_connected(&s,"left"){button_step(previous.0,&s.left,&s.input_mode,s.press_threshold,s.release_threshold)}else{[false;2]};
                    s.right_buttons=if s.status.is_none()||controller_connected(&s,"right"){button_step(previous.1,&s.right,&s.input_mode,s.press_threshold,s.release_threshold)}else{[false;2]};
                    let (left,right)=(touch_raw(&s.left),touch_raw(&s.right));
                    let snapshot=&mut *s;snapshot.engine.feed(0,left,now,&snapshot.touchpad);snapshot.engine.feed(1,right,now,&snapshot.touchpad);
                    send_driver(&mut s,now);
                    pulses=[new_press(previous.0,s.left_buttons),new_press(previous.1,s.right_buttons)];
                    amplitude=(s.haptic_amplitude*255.0).round() as u8;
                    if previous!=(s.left_buttons,s.right_buttons)||last_emit.elapsed()>=Duration::from_millis(33){let _=app.emit("stream-state",s.clone());last_emit=Instant::now()}
                }
                for (side,pulse) in pulses.into_iter().enumerate(){
                    if haptic_ready(pulse,pending_haptics[side],amplitude){
                        haptic_sequence=haptic_sequence.wrapping_add(1);
                        send_thumb_pulse(stream,side as u8,haptic_sequence,amplitude)?;
                        pending_haptics[side]=Some(haptic_sequence);
                    }
                }
            }
            _=>return Err(io::Error::new(io::ErrorKind::InvalidData,"Неизвестный формат потока"))
        }
    }
}
fn worker(app:AppHandle,state:Shared,generation:u64,transport:String){
    let mut owned_forward:Option<PathBuf>=None;
    loop {
        if state.generation.load(Ordering::SeqCst)!=generation {break}
        let update_required=state.snapshot.lock().map(|s|s.module_update_required).unwrap_or(false);
        if !update_required {publish(&app,&state,|s|{s.phase="searching".into();s.message=if transport=="usb"{"Поиск Quest через ADB…".into()}else{"Поиск Quest в локальной сети…".into()};s.status=None;s.endpoint=None;s.protocol=None;s.module_version=None;s.left=Sensor::default();s.right=Sensor::default();s.left_buttons=[false;2];s.right_buttons=[false;2]});}
        let endpoint=if transport=="usb"{
            match adb_forward(&app,&state){Ok((adb,owns))=>{if owns{owned_forward=Some(adb)};Ok(format!("127.0.0.1:{PORT}"))},Err(e)=>Err(e)}
        }else{discovery::discover(||state.generation.load(Ordering::SeqCst)!=generation).map_err(|e|e.to_string())};
        if state.generation.load(Ordering::SeqCst)!=generation {break}
        match endpoint {
            Ok(endpoint)=>{
                if !update_required {publish(&app,&state,|s|{s.phase="connecting".into();s.message="Ожидание потока модуля…".into();s.endpoint=Some(endpoint.clone())});}
                match endpoint.parse().ok().and_then(|addr|TcpStream::connect_timeout(&addr,Duration::from_secs(2)).ok()){
                    Some(mut stream)=>{
                        log::info!("TCP connected to {endpoint} ({transport})");
                        let result=stream.set_read_timeout(Some(Duration::from_millis(500)))
                            .and_then(|_|stream.set_write_timeout(Some(Duration::from_secs(2))))
                            .and_then(|_|read_frame(&mut stream,&app,&state,generation));
                        let _=stream.shutdown(std::net::Shutdown::Both);
                        if let Err(e)=result{if state.generation.load(Ordering::SeqCst)==generation{
                            log::warn!("TCP session {endpoint} ended: {e}; retrying");
                            if state.snapshot.lock().map(|s|s.module_update_required).unwrap_or(false) {
                                publish(&app,&state,|s|{s.phase="error".into();s.message=e.to_string();s.status=None});
                            } else {
                                publish(&app,&state,|s|{s.phase="searching".into();s.message=format!("Соединение прервано: {e}. Повторная попытка…");s.status=None;s.left=Sensor::default();s.right=Sensor::default();s.left_buttons=[false;2];s.right_buttons=[false;2]})
                            }
                        }}
                    }
                    None=>if !update_required {publish(&app,&state,|s|{s.phase="searching".into();s.message="Модуль пока не отвечает. Повторная попытка…".into()})},
                }
            }
            Err(e)=>if !update_required {publish(&app,&state,|s|{s.phase="searching".into();s.message=format!("{e}. Повторная попытка…")})},
        }
        let pause=if state.snapshot.lock().map(|s|s.module_update_required).unwrap_or(false){50}else{10};
        for _ in 0..pause{if state.generation.load(Ordering::SeqCst)!=generation{break}thread::sleep(Duration::from_millis(100))}
    }
    if let Some(path)=owned_forward{
        let next_is_usb=state.snapshot.lock().map(|s|s.transport=="usb").unwrap_or(false);
        if !next_is_usb {let _=adb(&path).args(["forward","--remove","tcp:27182"]).output();}
    }
}
fn transport_file(app:&AppHandle)->Option<PathBuf>{app.path().app_config_dir().ok().map(|d|d.join("transport"))}
fn config_file(app:&AppHandle)->Result<PathBuf,String>{app.path().app_config_dir().map(|d|d.join("config.json")).map_err(|e|e.to_string())}
fn read_user_config_file(file:&Path)->Option<UserConfig>{std::fs::read(file).ok().and_then(|bytes|serde_json::from_slice(&bytes).ok())}
fn write_user_config_file(file:&Path,config:&UserConfig)->Result<(),String>{
    std::fs::create_dir_all(file.parent().ok_or("Неверный путь конфигурации")?).map_err(|e|e.to_string())?;
    let json=serde_json::to_vec_pretty(config).map_err(|e|e.to_string())?;
    std::fs::write(file,json).map_err(|e|format!("Не удалось сохранить настройки: {e}"))?;
    // The driver reads this tiny flag before it launches the desktop application.
    std::fs::write(file.with_file_name("steamvr_lifecycle"),if config.steamvr_lifecycle{"1"}else{"0"})
        .map_err(|e|format!("Не удалось сохранить автозапуск SteamVR: {e}"))?;
    // The driver picks Classic or Fusion from this file when SteamVR starts.
    std::fs::write(file.with_file_name("steamvr_driver_mode"),&config.steamvr_driver_mode)
        .map_err(|e|format!("Не удалось сохранить режим драйвера SteamVR: {e}"))?;
    std::fs::write(file.with_file_name("steamvr_fusion_quest3"),&config.fusion_quest3)
        .map_err(|e|format!("Не удалось сохранить выбор контроллеров Quest 3: {e}"))
}
fn load_user_config(app:&AppHandle)->UserConfig{
    let mut config=config_file(app).ok().and_then(|f|read_user_config_file(&f)).unwrap_or_else(||{
        let mut config=UserConfig::default();
        if let Some(transport)=transport_file(app).and_then(|f|std::fs::read_to_string(f).ok()){config.transport=transport.trim().into()}
        config
    });
    if config.transport!="usb"&&config.transport!="lan"{config.transport="lan".into()}
    if !valid_mode(&config.input_mode){config.input_mode="touchpad".into()}
    if !valid_threshold(config.press_threshold){config.press_threshold=0.30}
    if !valid_threshold(config.release_threshold){config.release_threshold=0.20}
    if config.release_threshold>config.press_threshold{config.release_threshold=config.press_threshold}
    if !config.haptic_amplitude.is_finite()||!(0.0..=1.0).contains(&config.haptic_amplitude){config.haptic_amplitude=0.20}
    if !valid_language(&config.language){config.language="en".into()}
    if !valid_driver_mode(&config.steamvr_driver_mode){config.steamvr_driver_mode="classic".into()}
    if !valid_quest3_answer(&config.fusion_quest3){config.fusion_quest3="ask".into()}
    config.touchpad.sanitize();
    config
}
fn save_user_config(app:&AppHandle,config:&UserConfig)->Result<(),String>{
    let file=config_file(app)?;
    write_user_config_file(&file,config)
}
fn config_from_snapshot(s:&Snapshot)->UserConfig{UserConfig{transport:s.transport.clone(),input_mode:s.input_mode.clone(),press_threshold:s.press_threshold,release_threshold:s.release_threshold,haptic_amplitude:s.haptic_amplitude,language:s.language.clone(),steamvr_lifecycle:s.steamvr_lifecycle,steamvr_driver_mode:s.steamvr_driver_mode.clone(),fusion_quest3:s.fusion_quest3.clone(),touchpad:s.touchpad.clone()}}
fn apply_config(s:&mut Snapshot,config:UserConfig){
    s.transport=config.transport;s.input_mode=config.input_mode;s.press_threshold=config.press_threshold;s.release_threshold=config.release_threshold;s.haptic_amplitude=config.haptic_amplitude;
    s.language=config.language;s.steamvr_lifecycle=config.steamvr_lifecycle;s.steamvr_driver_mode=config.steamvr_driver_mode;s.fusion_quest3=config.fusion_quest3;s.touchpad=config.touchpad;
}
fn start_worker(app:&AppHandle,state:&Shared,transport:String){
    let generation=state.generation.fetch_add(1,Ordering::SeqCst)+1;
    publish(app,state,|s|{s.engine.release();let mut config=config_from_snapshot(s);config.transport=transport.clone();let steamvr_driver=s.steamvr_driver.clone();*s=Snapshot{steamvr_driver,..Snapshot::default()};apply_config(s,config)});
    let (app,shared)=(app.clone(),state.clone());thread::spawn(move||worker(app,shared,generation,transport));
}
#[tauri::command]
fn get_stream_state(state:State<Shared>)->Snapshot{state.snapshot.lock().unwrap().clone()}
#[tauri::command]
fn set_transport(app:AppHandle,state:State<Shared>,transport:String)->Result<(),String>{
    if transport!="usb"&&transport!="lan"{return Err("Неизвестный способ подключения".into())}
    let _guard=state.config_lock.lock().map_err(|e|e.to_string())?;
    if state.snapshot.lock().map(|s|s.transport==transport).unwrap_or(false){return Ok(())}
    let mut config=state.snapshot.lock().map(|s|config_from_snapshot(&s)).map_err(|e|e.to_string())?;
    config.transport=transport.clone();save_user_config(&app,&config)?;
    start_worker(&app,state.inner(),transport);Ok(())
}
#[tauri::command]
fn set_input_mode(app:AppHandle,state:State<Shared>,mode:String)->Result<(),String>{
    if !valid_mode(&mode){return Err("Неизвестный режим".into())}
    let _guard=state.config_lock.lock().map_err(|e|e.to_string())?;
    let mut config=state.snapshot.lock().map(|s|config_from_snapshot(&s)).map_err(|e|e.to_string())?;
    if config.input_mode==mode{return Ok(())}
    config.input_mode=mode.clone();save_user_config(&app,&config)?;
    publish(&app,state.inner(),|s|{s.input_mode=mode;s.left_buttons=[false;2];s.right_buttons=[false;2]});Ok(())
}
#[tauri::command]
fn set_button_thresholds(app:AppHandle,state:State<Shared>,press:f32,release:f32)->Result<(),String>{
    if !valid_threshold(press)||!valid_threshold(release)||release>press{return Err("Пороги должны быть в пределах 0–1; отпускание не выше нажатия".into())}
    let _guard=state.config_lock.lock().map_err(|e|e.to_string())?;
    let press=(press*100.0).round()/100.0;
    let release=(release*100.0).round()/100.0;
    let mut config=state.snapshot.lock().map(|s|config_from_snapshot(&s)).map_err(|e|e.to_string())?;
    if (config.press_threshold-press).abs()<0.001&&(config.release_threshold-release).abs()<0.001{return Ok(())}
    config.press_threshold=press;config.release_threshold=release;save_user_config(&app,&config)?;
    publish(&app,state.inner(),|s|{s.press_threshold=press;s.release_threshold=release});Ok(())
}
#[tauri::command]
fn set_haptic_amplitude(app:AppHandle,state:State<Shared>,amplitude:f32)->Result<(),String>{
    if !amplitude.is_finite()||!(0.0..=1.0).contains(&amplitude){return Err("Мощность вибрации должна быть от 0 до 1".into())}
    let _guard=state.config_lock.lock().map_err(|e|e.to_string())?;
    let amplitude=(amplitude*100.0).round()/100.0;
    let mut config=state.snapshot.lock().map(|s|config_from_snapshot(&s)).map_err(|e|e.to_string())?;
    if (config.haptic_amplitude-amplitude).abs()<0.001{return Ok(())}
    config.haptic_amplitude=amplitude;save_user_config(&app,&config)?;
    publish(&app,state.inner(),|s|s.haptic_amplitude=amplitude);Ok(())
}
#[tauri::command]
fn set_language(app:AppHandle,state:State<Shared>,language:String)->Result<(),String>{
    if !valid_language(&language){return Err("Неизвестный язык".into())}
    let _guard=state.config_lock.lock().map_err(|e|e.to_string())?;
    let mut config=state.snapshot.lock().map(|s|config_from_snapshot(&s)).map_err(|e|e.to_string())?;
    if config.language==language{return Ok(())}
    config.language=language.clone();save_user_config(&app,&config)?;
    if let Some(tray)=app.try_state::<Tray>(){let (show,quit)=tray_labels(&language);let _=tray.show.set_text(show);let _=tray.quit.set_text(quit);}
    publish(&app,state.inner(),|s|s.language=language);Ok(())
}
#[tauri::command]
fn set_steamvr_lifecycle(app:AppHandle,state:State<Shared>,enabled:bool)->Result<(),String>{
    let _guard=state.config_lock.lock().map_err(|e|e.to_string())?;
    let mut config=state.snapshot.lock().map(|s|config_from_snapshot(&s)).map_err(|e|e.to_string())?;
    if config.steamvr_lifecycle==enabled{return Ok(())}
    config.steamvr_lifecycle=enabled;
    save_user_config(&app,&config)?;
    publish(&app,state.inner(),|s|s.steamvr_lifecycle=enabled);
    Ok(())
}
#[tauri::command]
fn set_steamvr_driver_mode(app:AppHandle,state:State<Shared>,mode:String)->Result<(),String>{
    if !valid_driver_mode(&mode){return Err("Неизвестный режим драйвера".into())}
    let _guard=state.config_lock.lock().map_err(|e|e.to_string())?;
    let mut config=state.snapshot.lock().map(|s|config_from_snapshot(&s)).map_err(|e|e.to_string())?;
    if config.steamvr_driver_mode==mode{return Ok(())}
    config.steamvr_driver_mode=mode.clone();save_user_config(&app,&config)?;
    publish(&app,state.inner(),|s|s.steamvr_driver_mode=mode);Ok(())
}
#[tauri::command]
fn set_fusion_quest3(app:AppHandle,state:State<Shared>,answer:String)->Result<(),String>{
    if !valid_quest3_answer(&answer){return Err("Неизвестный ответ".into())}
    let _guard=state.config_lock.lock().map_err(|e|e.to_string())?;
    let mut config=state.snapshot.lock().map(|s|config_from_snapshot(&s)).map_err(|e|e.to_string())?;
    if config.fusion_quest3==answer{return Ok(())}
    config.fusion_quest3=answer.clone();save_user_config(&app,&config)?;
    publish(&app,state.inner(),|s|s.fusion_quest3=answer);Ok(())
}
#[tauri::command]
fn set_touchpad_settings(app:AppHandle,state:State<Shared>,settings:touch::Settings)->Result<(),String>{
    let settings=settings.checked()?;
    let _guard=state.config_lock.lock().map_err(|e|e.to_string())?;
    let mut config=state.snapshot.lock().map(|s|config_from_snapshot(&s)).map_err(|e|e.to_string())?;
    if config.touchpad==settings{return Ok(())}
    config.touchpad=settings.clone();save_user_config(&app,&config)?;
    publish(&app,state.inner(),|s|s.touchpad=settings);Ok(())
}
// Time alone moves an undecided contact or a gliding swipe; a 100 ms heartbeat keeps the driver from going stale.
fn watch_touch(app:AppHandle,state:Shared){
    thread::spawn(move||{
        let mut last_emit=Instant::now();
        loop{
            thread::sleep(Duration::from_millis(8));
            let now=touch::now_us();
            let Ok(mut s)=state.snapshot.lock() else {continue};
            if !s.engine.busy(now,&s.touchpad)&&now.saturating_sub(s.engine.last_sent)<100_000 {continue}
            let before=(s.left_pad,s.right_pad,s.steamvr_connected,s.steamvr_driver.clone());
            send_driver(&mut s,now);
            if before!=(s.left_pad,s.right_pad,s.steamvr_connected,s.steamvr_driver.clone())&&last_emit.elapsed()>=Duration::from_millis(33){let _=app.emit("stream-state",s.clone());last_emit=Instant::now()}
        }
    });
}
fn watch_steamvr_lifecycle(app:AppHandle,state:Shared){
    thread::spawn(move||{
        let mut was_connected=false;
        let mut lost_at:Option<Instant>=None;
        loop{
            thread::sleep(Duration::from_millis(500));
            let Ok(snapshot)=state.snapshot.lock() else {continue};
            if !snapshot.steamvr_lifecycle {was_connected=false;lost_at=None;continue}
            if snapshot.steamvr_connected {was_connected=true;lost_at=None;continue}
            if was_connected {
                let since=lost_at.get_or_insert_with(Instant::now);
                if since.elapsed()>=Duration::from_secs(5){drop(snapshot);app.exit(0);return}
            }
        }
    });
}
#[tauri::command]
fn open_module_page()->Result<(),String>{
    open_external_page("https://github.com/Lateir/qptp-module")
}
#[tauri::command]
fn open_release_page(target:&str)->Result<(),String>{
    let url=match target {
        "module"=>"https://github.com/Lateir/qptp-module/releases/latest",
        "app"=>"https://github.com/Lateir/qptp/releases/latest",
        _=>return Err("Неизвестный тип обновления".into()),
    };
    open_external_page(url)
}
fn open_external_page(url:&str)->Result<(),String>{
    let wide:Vec<u16>=url.encode_utf16().chain(std::iter::once(0)).collect();
    let result=unsafe{windows_sys::Win32::UI::Shell::ShellExecuteW(
        std::ptr::null_mut(),std::ptr::null(),wide.as_ptr(),std::ptr::null(),std::ptr::null(),1
    )};
    if (result as isize)<=32 {Err(format!("Не удалось открыть браузер (код {})",result as isize))}else{Ok(())}
}
fn show_window(app:&AppHandle){
    save_tray_state(app,false);
    if let Some(w)=app.get_webview_window("main"){let _=w.unminimize();let _=w.show();let _=w.set_focus();return}
    // Building a webview inside an event handler can deadlock on Windows, so do it off the main thread.
    let app=app.clone();
    thread::spawn(move||{if let Some(cfg)=app.config().app.windows.first(){if let Ok(w)=tauri::WebviewWindowBuilder::from_config(&app,cfg).and_then(|b|b.build()){let _=w.set_focus();}}});
}
fn tray_state_file(app:&AppHandle)->Option<PathBuf>{app.path().app_config_dir().ok().map(|d|d.join("window_in_tray"))}
fn starts_in_tray(app:&AppHandle)->bool{tray_state_file(app).and_then(|p|std::fs::read_to_string(p).ok()).is_some_and(|s|s.trim()=="1")}
fn save_tray_state(app:&AppHandle,in_tray:bool){
    if let Some(path)=tray_state_file(app){
        if let Some(parent)=path.parent(){let _=std::fs::create_dir_all(parent);}
        let _=std::fs::write(path,if in_tray{"1"}else{"0"});
    }
}
fn tray_labels(language:&str)->(&'static str,&'static str){
    match language {
        "en"=>("Open","Quit"),"zh-CN"=>("打开","退出"),"hi"=>("खोलें","बाहर निकलें"),
        "es"=>("Abrir","Salir"),"ar"=>("فتح","خروج"),"fr"=>("Ouvrir","Quitter"),
        "bn"=>("খুলুন","বন্ধ করুন"),"pt-BR"=>("Abrir","Sair"),"id"=>("Buka","Keluar"),
        "ja"=>("開く","終了"),"de"=>("Öffnen","Beenden"),_=>("Открыть","Выход"),
    }
}
fn setup_tray(app:&AppHandle,language:&str)->tauri::Result<()>{
    let controllers=MenuItem::with_id(app,"controllers","○ L —    ○ R —",true,None::<&str>)?;
    let (show_label,quit_label)=tray_labels(language);
    let show=MenuItem::with_id(app,"show",show_label,true,None::<&str>)?;
    let quit=MenuItem::with_id(app,"quit",quit_label,true,None::<&str>)?;
    let menu=Menu::with_items(app,&[&controllers,&PredefinedMenuItem::separator(app)?,&show,&quit])?;
    let mut tray=TrayIconBuilder::with_id("main").tooltip("Quest Pro Touch Plus").menu(&menu).show_menu_on_left_click(false)
        .on_menu_event(|app,e|match e.id().as_ref(){"show"=>show_window(app),"quit"=>app.exit(0),_=>{}})
        .on_tray_icon_event(|tray,e|if let TrayIconEvent::Click{button:MouseButton::Left,button_state:MouseButtonState::Up,..}=e{show_window(tray.app_handle())});
    if let Some(icon)=app.default_window_icon(){tray=tray.icon(icon.clone())}
    tray.build(app)?;
    app.manage(Tray{controllers,show,quit,last:Mutex::new(None)});
    Ok(())
}
pub fn run(){
    let state:Shared=Arc::new(StreamState{snapshot:Mutex::new(Snapshot::default()),config_lock:Mutex::new(()),generation:AtomicU64::new(0),adb_used:AtomicBool::new(false)});
    tauri::Builder::default().manage(state)
        .setup(|app|{
            let handle=app.handle().clone();
            let config=load_user_config(&handle);
            setup_tray(&handle,&config.language)?;
            let _=save_user_config(&handle,&config);
            let transport=config.transport.clone();
            if let Ok(mut snapshot)=handle.state::<Shared>().snapshot.lock(){apply_config(&mut snapshot,config);}
            start_worker(&handle,handle.state::<Shared>().inner(),transport);
            watch_steamvr_lifecycle(handle.clone(),handle.state::<Shared>().inner().clone());
            watch_touch(handle.clone(),handle.state::<Shared>().inner().clone());
            #[cfg(all(windows, not(debug_assertions)))]
            thread::spawn(||{let _=steamvr_install::ensure_registered();});
            if starts_in_tray(&handle){
                if let Some(window)=handle.get_webview_window("main"){let _=window.hide();}
            }else{show_window(&handle);}
            Ok(())
        })
        .on_window_event(|window,event|match event{
            // Minimize = hide to tray: destroy the window so WebView2 releases its memory.
            WindowEvent::Resized(_) if window.is_minimized().unwrap_or(false)=>{save_tray_state(window.app_handle(),true);let _=window.destroy();}
            WindowEvent::CloseRequested{..}=>window.app_handle().exit(0),
            _=>{}
        })
        .invoke_handler(tauri::generate_handler![get_stream_state,set_transport,set_input_mode,set_button_thresholds,set_haptic_amplitude,set_language,set_steamvr_lifecycle,set_steamvr_driver_mode,set_fusion_quest3,set_touchpad_settings,open_module_page,open_release_page])
        .build(tauri::generate_context!()).expect("tauri application error")
        .run(|app,event|match event{
            // Destroying the last window (tray mode) must not quit the app; explicit exit passes a code.
            RunEvent::ExitRequested{api,code:None,..}=>api.prevent_exit(),
            RunEvent::Exit=>{
                let state=app.state::<Shared>();
                if let Ok(mut snapshot)=state.snapshot.lock(){snapshot.engine.release();}
                if state.adb_used.load(Ordering::SeqCst){if let Ok(path)=adb_path(app){let _=adb(&path).arg("kill-server").output();}}
            }
            _=>{}
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tcp_pair()->(TcpStream,TcpStream) {
        let listener=std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let client=TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (server,_)=listener.accept().unwrap();
        client.set_read_timeout(Some(Duration::from_millis(20))).unwrap();
        (client,server)
    }
    #[test]
    fn silent_and_partial_streams_expire_then_new_connection_works() {
        for partial in [false,true] {
            let (mut client,mut server)=tcp_pair();
            if partial {server.write_all(b"QP").unwrap()}
            let start=Instant::now();
            let error=read_exact_deadline(&mut client,&mut [0;4],Duration::from_millis(100),||false).unwrap_err();
            assert_eq!(error.kind(),io::ErrorKind::TimedOut);
            assert!(start.elapsed()<Duration::from_secs(1));
            client.shutdown(std::net::Shutdown::Both).unwrap();
            let (mut client,mut server)=tcp_pair();
            server.write_all(b"QPR3").unwrap();
            let mut header=[0;4];
            read_exact_deadline(&mut client,&mut header,Duration::from_secs(1),||false).unwrap();
            assert_eq!(&header,b"QPR3");
        }
    }
    #[test]
    fn read_deadline_preserves_fragments_and_cancellation() {
        let (mut client,mut server)=tcp_pair();
        let sender=thread::spawn(move|| {
            server.write_all(b"QP").unwrap();thread::sleep(Duration::from_millis(60));
            server.write_all(b"R3").unwrap();
        });
        let mut header=[0;4];
        read_exact_deadline(&mut client,&mut header,Duration::from_secs(1),||false).unwrap();
        assert_eq!(&header,b"QPR3");sender.join().unwrap();
        assert_eq!(read_exact_deadline(&mut client,&mut header,Duration::from_secs(1),||true).unwrap_err().kind(),io::ErrorKind::Interrupted);
    }
    fn sample(y:u16,force:f32)->Sensor{Sensor{x:80,y,force,..Sensor::default()}}
    #[test]
    fn tray_shows_both_controller_batteries_on_one_line(){
        let snapshot=Snapshot{phase:"connected".into(),status:Some(serde_json::json!({
            "controllers":{
                "left":{"connected":true,"battery_percent":78.6},
                "right":{"connected":true,"battery":42}
            }
        })),..Snapshot::default()};
        assert_eq!(tray_label(tray_status(&snapshot)),"✓ L 79%    ✓ R 42%");
        let disconnected=Snapshot{phase:"searching".into(),..snapshot};
        assert_eq!(tray_label(tray_status(&disconnected)),"○ L —    ○ R —");
    }
    #[test]
    fn button_hysteresis(){
        let idle=[false;2];
        assert_eq!(button_step(idle,&sample(80,0.29),"button",0.30,0.20),idle);
        let pressed=button_step(idle,&sample(80,0.30),"button",0.30,0.20);
        assert_eq!(pressed,[true,false]);
        assert_eq!(button_step(pressed,&sample(80,0.20),"button",0.30,0.20),pressed);
        assert_eq!(button_step(pressed,&sample(80,0.19),"button",0.30,0.20),idle);
        assert_eq!(button_step(idle,&sample(80,0.0),"button",0.0,0.0),[true,false]);
        assert_eq!(button_step(idle,&sample(80,1.0),"button",1.0,1.0),[true,false]);
    }
    #[test]
    fn two_buttons_latch_the_initial_half(){
        let left=button_step([false;2],&sample(127,0.35),"two_buttons",0.30,0.20);
        assert_eq!(left,[true,false]);
        assert_eq!(button_step(left,&sample(200,0.35),"two_buttons",0.30,0.20),left);
        assert_eq!(button_step([false;2],&sample(128,0.35),"two_buttons",0.30,0.20),[false,true]);
        assert_eq!(button_step(left,&Sensor{x:0,y:255,force:0.35,..Sensor::default()},"two_buttons",0.30,0.20),[false;2]);
        assert_eq!(button_step(left,&sample(80,0.35),"touchpad",0.30,0.20),[false;2]);
    }
    #[test]
    fn config_defaults_missing_fields(){
        let config:UserConfig=serde_json::from_str("{}").unwrap();
        assert_eq!(config.transport,"lan");
        assert_eq!(config.input_mode,"touchpad");
        assert_eq!(config.press_threshold,0.30);
        assert_eq!(config.release_threshold,0.20);
        assert_eq!(config.haptic_amplitude,0.20);
        assert_eq!(config.language,system_language());
        // Configs saved before Fusion keep the Treadmill driver and an identity touchpad.
        let old:UserConfig=serde_json::from_str(r#"{"transport":"usb","inputMode":"button","steamvrLifecycle":false}"#).unwrap();
        assert_eq!(old.steamvr_driver_mode,"classic");
        assert_eq!(old.fusion_quest3,"ask");
        assert_eq!(old.touchpad,touch::Settings::default());
    }
    #[test]
    fn qpr4_sides_carry_contact_size_and_flag(){
        let mut side=[0u8;28];
        side[0..2].copy_from_slice(&200u16.to_le_bytes());side[2..4].copy_from_slice(&55u16.to_le_bytes());
        for (offset,value) in [(4,0.5f32),(8,80.0),(12,1.0),(16,0.25),(20,0.75)]{side[offset..offset+4].copy_from_slice(&value.to_le_bytes())}
        side[24]=1;
        let sensor=sensor4(&side);
        assert_eq!((sensor.x,sensor.y,sensor.force,sensor.size,sensor.touch),(200,200,0.5,Some(80.0),Some(true)));
        assert_eq!((sensor.stylus,sensor.trigger_proximity,sensor.trigger_slide),(1.0,0.25,0.75));
        side[24]=2;assert_eq!(sensor4(&side).touch,Some(false));
    }
    #[test]
    fn system_language_matching_and_english_fallback(){
        assert_eq!(choose_language(["ru-RU"]),"ru");
        assert_eq!(choose_language(["zh-Hans-CN"]),"zh-CN");
        assert_eq!(choose_language(["pt-PT"]),"pt-BR");
        assert_eq!(choose_language(["ko-KR","ja-JP"]),"ja");
        assert_eq!(choose_language(["zh-TW","ko-KR"]),"en");
    }
    #[test]
    fn haptics_only_on_new_press_and_one_outstanding_request_per_controller(){
        assert!(new_press([false;2],[true,false]));
        assert!(!new_press([true,false],[false;2]));
        assert!(!new_press([true,false],[false,true]));
        let mut pending=[Some(7),Some(8)];
        assert!(!haptic_ready(true,pending[0],51));
        assert!(!haptic_ready(true,None,0));
        assert!(!haptic_ready(false,None,51));
        acknowledge_haptic(&mut pending,8);
        assert_eq!(pending,[Some(7),None]);
        assert!(haptic_ready(true,pending[1],51));
        acknowledge_haptic(&mut pending,7);
        assert_eq!(pending,[None,None]);
    }
    #[test]
    fn config_round_trip(){
        let dir=std::env::temp_dir().join(format!("qptp-config-test-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let file=dir.join("config.json");
        let touchpad=touch::Settings{style:"swipe".into(),swipe_gain:0.5,..touch::Settings::default()};
        let config=UserConfig{transport:"lan".into(),input_mode:"two_buttons".into(),press_threshold:0.45,release_threshold:0.12,haptic_amplitude:0.65,language:"ja".into(),steamvr_lifecycle:false,steamvr_driver_mode:"fusion".into(),fusion_quest3:"pro".into(),touchpad:touchpad.clone()};
        write_user_config_file(&file,&config).unwrap();
        let loaded=read_user_config_file(&file).unwrap();
        assert_eq!(loaded.transport,config.transport);
        assert_eq!(loaded.input_mode,config.input_mode);
        assert_eq!(loaded.press_threshold,config.press_threshold);
        assert_eq!(loaded.release_threshold,config.release_threshold);
        assert_eq!(loaded.haptic_amplitude,config.haptic_amplitude);
        assert_eq!(loaded.language,config.language);
        assert!(!loaded.steamvr_lifecycle);
        assert_eq!(std::fs::read_to_string(dir.join("steamvr_lifecycle")).unwrap(),"0");
        assert_eq!(loaded.touchpad,touchpad);
        assert_eq!(std::fs::read_to_string(dir.join("steamvr_driver_mode")).unwrap(),"fusion");
        assert_eq!(std::fs::read_to_string(dir.join("steamvr_fusion_quest3")).unwrap(),"pro");
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn supported_languages_are_validated(){
        for language in ["ru","en","zh-CN","hi","es","ar","fr","bn","pt-BR","id","ja","de"]{assert!(valid_language(language));}
        assert!(!valid_language("xx"));
    }
}
