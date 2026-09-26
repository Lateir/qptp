use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{io::{self, Read, Write}, net::{TcpStream, UdpSocket}, path::{Path, PathBuf}, process::Command, sync::{atomic::{AtomicBool, AtomicU64, Ordering}, Arc, Mutex}, thread, time::{Duration, Instant}};
use tauri::{image::Image, menu::{IconMenuItem, Menu, MenuItem, PredefinedMenuItem}, tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent}, AppHandle, Emitter, Manager, RunEvent, State, WindowEvent, Wry};
mod extra_ipc;
#[cfg(windows)]
pub mod steamvr_install;

const PORT: u16 = 27182;
const DISCOVERY_PORT: u16 = 27183;
const RELEASE_THRESHOLD: f32 = 0.20;
const HAPTIC_DURATION_MS: u16 = 2;

#[derive(Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
struct UserConfig { transport: String, input_mode: String, press_threshold: f32, haptic_amplitude: f32, language: String, steamvr_lifecycle: bool }
impl Default for UserConfig {
    fn default()->Self { Self { transport:"lan".into(), input_mode:"touchpad".into(), press_threshold:0.30, haptic_amplitude:0.20, language:system_language().into(), steamvr_lifecycle:true } }
}
fn valid_mode(mode:&str)->bool {matches!(mode,"touchpad"|"button"|"two_buttons")}
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
fn valid_threshold(value:f32)->bool {value.is_finite()&&value>RELEASE_THRESHOLD&&value<=1.0}

#[derive(Clone, Default, Serialize)]
struct Sensor { x: u16, y: u16, force: f32 }
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    phase: String, message: String, transport: String, endpoint: Option<String>,
    protocol: Option<String>, samples: u64, left: Sensor, right: Sensor, status: Option<Value>,
    input_mode: String, press_threshold: f32, haptic_amplitude: f32, language: String, left_buttons: [bool;2], right_buttons: [bool;2],
    steamvr_connected: bool, steamvr_lifecycle: bool,
}
impl Default for Snapshot {
    fn default() -> Self { Self { phase:"searching".into(), message:"Поиск устройства…".into(), transport:"lan".into(), endpoint:None, protocol:None, samples:0, left:Sensor::default(), right:Sensor::default(), status:None,input_mode:"touchpad".into(),press_threshold:0.30,haptic_amplitude:0.20,language:"ru".into(),left_buttons:[false;2],right_buttons:[false;2],steamvr_connected:false,steamvr_lifecycle:true } }
}
struct StreamState { snapshot: Mutex<Snapshot>, config_lock: Mutex<()>, generation: AtomicU64, adb_used: AtomicBool }
type Shared = Arc<StreamState>;
struct Tray { left: IconMenuItem<Wry>, right: IconMenuItem<Wry>, show: MenuItem<Wry>, quit: MenuItem<Wry>, last: Mutex<Option<(bool,bool)>> }

fn controller_connected(s:&Snapshot, side:&str)->bool {
    if s.phase!="connected" {return false}
    let Some(status)=&s.status else {return false};
    let value=status.get(side).or_else(||status.get("controllers").and_then(|c|c.get(side)));
    value.and_then(|v|v.get("connected")).and_then(Value::as_bool).unwrap_or(false)
}
fn dot(on:bool)->Image<'static>{
    let (r,g,b)=if on{(34,197,94)}else{(128,136,150)};
    let mut px=Vec::with_capacity(16*16*4);
    for y in 0..16{for x in 0..16{let d=((x as f32-7.5).powi(2)+(y as f32-7.5).powi(2)).sqrt();px.extend_from_slice(&[r,g,b,((6.5-d).clamp(0.0,1.0)*255.0) as u8])}}
    Image::new_owned(px,16,16)
}
fn update_tray(app:&AppHandle, lr:(bool,bool)){
    let Some(tray)=app.try_state::<Tray>() else {return};
    let mut last=tray.last.lock().unwrap();
    if *last==Some(lr) {return}
    *last=Some(lr);
    let _=tray.left.set_icon(Some(dot(lr.0)));let _=tray.right.set_icon(Some(dot(lr.1)));
}
fn publish(app:&AppHandle, state:&Shared, change:impl FnOnce(&mut Snapshot)) {
    // Tray is updated after the lock is released: menu calls block on the main thread.
    let lr=match state.snapshot.lock() {Ok(mut snapshot)=>{change(&mut snapshot);snapshot.steamvr_connected=extra_ipc::publish(&snapshot);let _=app.emit("stream-state",snapshot.clone());(controller_connected(&snapshot,"left"),controller_connected(&snapshot,"right"))},Err(_)=>return};
    update_tray(app,lr);
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
fn discover()->io::Result<String>{
    let socket=UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;socket.set_read_timeout(Some(Duration::from_millis(250)))?;
    let start=Instant::now();let mut next=Instant::now();
    let mut packet=[0u8;64];
    while start.elapsed()<Duration::from_secs(3){
        if Instant::now()>=next {let _=socket.send_to(b"QPD1",("255.255.255.255",DISCOVERY_PORT));next=Instant::now()+Duration::from_millis(700)}
        if let Ok((n,peer))=socket.recv_from(&mut packet) {
            if n==8 && &packet[..4]==b"QPO1" && packet[6..8]==[1,0] {
                let port=u16::from_le_bytes([packet[4],packet[5]]);
                if port>=1024 {return Ok(format!("{}:{port}",peer.ip()))}
            }
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound,"Quest не найден в локальной сети"))
}
fn read_exact_checked(stream:&mut TcpStream, bytes:&mut [u8], state:&Shared, generation:u64)->io::Result<()> {
    let mut offset=0;
    while offset<bytes.len() {
        if state.generation.load(Ordering::SeqCst)!=generation {return Err(io::Error::new(io::ErrorKind::Interrupted,"Отключено"))}
        match stream.read(&mut bytes[offset..]) {
            Ok(0)=>return Err(io::Error::new(io::ErrorKind::UnexpectedEof,"Поток закрыт")),
            Ok(n)=>offset+=n,
            Err(e) if matches!(e.kind(),io::ErrorKind::WouldBlock|io::ErrorKind::TimedOut)=>{},
            Err(e)=>return Err(e),
        }
    }
    Ok(())
}
fn sensor(b:&[u8])->Sensor{
    // Module's Y axis is inverted relative to the pad; flip so 0 is the top.
    Sensor{x:u16::from_le_bytes([b[0],b[1]]),y:255u16.saturating_sub(u16::from_le_bytes([b[2],b[3]])),force:f32::from_le_bytes(b[4..8].try_into().unwrap())}
}
fn button_step(previous:[bool;2], sensor:&Sensor, mode:&str, press_threshold:f32)->[bool;2]{
    // Raw (0, 0) is no touch. The decoded Y axis is flipped, so idle is (0, 255).
    if mode=="touchpad" || (sensor.x==0&&sensor.y==255) || !sensor.force.is_finite() || sensor.force<=RELEASE_THRESHOLD {return [false;2]}
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
                publish(app,state,|s|{s.status=Some(status);s.phase="connected".into();s.message="Поток сенсоров активен".into();s.protocol=Some("QPR2 / QPS1".into());if !controller_connected(s,"left"){s.left_buttons=[false;2]}if !controller_connected(s,"right"){s.right_buttons=[false;2]}});
            }
            b"QPR2"=>{
                let mut body=[0u8;28];read_exact_checked(stream,&mut body,state,generation)?;
                if state.generation.load(Ordering::SeqCst)!=generation {return Ok(())}
                let (left,right)=(sensor(&body[12..20]),sensor(&body[20..28]));
                let mut pulses=[false;2];let mut amplitude=0u8;
                if let Ok(mut s)=state.snapshot.lock(){
                    let previous=(s.left_buttons,s.right_buttons);
                    s.left=left;s.right=right;s.samples+=1;s.phase="connected".into();s.message="Поток сенсоров активен".into();s.protocol=Some("QPR2 / QPS1".into());
                    s.left_buttons=if s.status.is_none()||controller_connected(&s,"left"){button_step(previous.0,&s.left,&s.input_mode,s.press_threshold)}else{[false;2]};
                    s.right_buttons=if s.status.is_none()||controller_connected(&s,"right"){button_step(previous.1,&s.right,&s.input_mode,s.press_threshold)}else{[false;2]};
                    s.steamvr_connected=extra_ipc::publish(&s);
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
        publish(&app,&state,|s|{s.phase="searching".into();s.message=if transport=="usb"{"Поиск Quest через ADB…".into()}else{"Поиск Quest в локальной сети…".into()};s.status=None;s.endpoint=None;s.protocol=None;s.left=Sensor::default();s.right=Sensor::default();s.left_buttons=[false;2];s.right_buttons=[false;2]});
        let endpoint=if transport=="usb"{
            match adb_forward(&app,&state){Ok((adb,owns))=>{if owns{owned_forward=Some(adb)};Ok(format!("127.0.0.1:{PORT}"))},Err(e)=>Err(e)}
        }else{discover().map_err(|e|e.to_string())};
        if state.generation.load(Ordering::SeqCst)!=generation {break}
        match endpoint {
            Ok(endpoint)=>{
                publish(&app,&state,|s|{s.phase="connecting".into();s.message="Ожидание потока модуля…".into();s.endpoint=Some(endpoint.clone())});
                match endpoint.parse().ok().and_then(|addr|TcpStream::connect_timeout(&addr,Duration::from_secs(2)).ok()){
                    Some(mut stream)=>{
                        let _=stream.set_read_timeout(Some(Duration::from_millis(500)));
                        if let Err(e)=read_frame(&mut stream,&app,&state,generation){if state.generation.load(Ordering::SeqCst)==generation{publish(&app,&state,|s|{s.phase="searching".into();s.message=format!("Соединение прервано: {e}. Повторная попытка…");s.status=None})}}
                    }
                    None=>publish(&app,&state,|s|{s.phase="searching".into();s.message="Модуль пока не отвечает. Повторная попытка…".into()}),
                }
            }
            Err(e)=>publish(&app,&state,|s|{s.phase="searching".into();s.message=format!("{e}. Повторная попытка…")}),
        }
        for _ in 0..10{if state.generation.load(Ordering::SeqCst)!=generation{break}thread::sleep(Duration::from_millis(100))}
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
        .map_err(|e|format!("Не удалось сохранить автозапуск SteamVR: {e}"))
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
    if !config.haptic_amplitude.is_finite()||!(0.0..=1.0).contains(&config.haptic_amplitude){config.haptic_amplitude=0.20}
    if !valid_language(&config.language){config.language="en".into()}
    config
}
fn save_user_config(app:&AppHandle,config:&UserConfig)->Result<(),String>{
    let file=config_file(app)?;
    write_user_config_file(&file,config)
}
fn config_from_snapshot(s:&Snapshot)->UserConfig{UserConfig{transport:s.transport.clone(),input_mode:s.input_mode.clone(),press_threshold:s.press_threshold,haptic_amplitude:s.haptic_amplitude,language:s.language.clone(),steamvr_lifecycle:s.steamvr_lifecycle}}
fn start_worker(app:&AppHandle,state:&Shared,transport:String){
    let generation=state.generation.fetch_add(1,Ordering::SeqCst)+1;
    publish(app,state,|s|{let input_mode=s.input_mode.clone();let press_threshold=s.press_threshold;let haptic_amplitude=s.haptic_amplitude;let language=s.language.clone();let steamvr_lifecycle=s.steamvr_lifecycle;*s=Snapshot{transport:transport.clone(),input_mode,press_threshold,haptic_amplitude,language,steamvr_lifecycle,..Snapshot::default()}});
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
fn set_press_threshold(app:AppHandle,state:State<Shared>,threshold:f32)->Result<(),String>{
    if !valid_threshold(threshold){return Err("Порог нажатия должен быть больше 0.20 и не больше 1.00".into())}
    let _guard=state.config_lock.lock().map_err(|e|e.to_string())?;
    let threshold=(threshold*100.0).round()/100.0;
    let mut config=state.snapshot.lock().map(|s|config_from_snapshot(&s)).map_err(|e|e.to_string())?;
    if (config.press_threshold-threshold).abs()<0.001{return Ok(())}
    config.press_threshold=threshold;save_user_config(&app,&config)?;
    publish(&app,state.inner(),|s|s.press_threshold=threshold);Ok(())
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
    const URL:&str="https://github.com/Lateir/qptp-module";
    let wide:Vec<u16>=URL.encode_utf16().chain(std::iter::once(0)).collect();
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
    let left=IconMenuItem::with_id(app,"left","L",true,Some(dot(false)),None::<&str>)?;
    let right=IconMenuItem::with_id(app,"right","R",true,Some(dot(false)),None::<&str>)?;
    let (show_label,quit_label)=tray_labels(language);
    let show=MenuItem::with_id(app,"show",show_label,true,None::<&str>)?;
    let quit=MenuItem::with_id(app,"quit",quit_label,true,None::<&str>)?;
    let menu=Menu::with_items(app,&[&left,&right,&PredefinedMenuItem::separator(app)?,&show,&quit])?;
    let mut tray=TrayIconBuilder::with_id("main").tooltip("Quest Pro Touch Plus").menu(&menu).show_menu_on_left_click(false)
        .on_menu_event(|app,e|match e.id().as_ref(){"show"=>show_window(app),"quit"=>app.exit(0),_=>{}})
        .on_tray_icon_event(|tray,e|if let TrayIconEvent::Click{button:MouseButton::Left,button_state:MouseButtonState::Up,..}=e{show_window(tray.app_handle())});
    if let Some(icon)=app.default_window_icon(){tray=tray.icon(icon.clone())}
    tray.build(app)?;
    app.manage(Tray{left,right,show,quit,last:Mutex::new(None)});
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
            if let Ok(mut snapshot)=handle.state::<Shared>().snapshot.lock(){snapshot.input_mode=config.input_mode;snapshot.press_threshold=config.press_threshold;snapshot.haptic_amplitude=config.haptic_amplitude;snapshot.language=config.language;snapshot.steamvr_lifecycle=config.steamvr_lifecycle;}
            start_worker(&handle,handle.state::<Shared>().inner(),transport);
            watch_steamvr_lifecycle(handle.clone(),handle.state::<Shared>().inner().clone());
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
        .invoke_handler(tauri::generate_handler![get_stream_state,set_transport,set_input_mode,set_press_threshold,set_haptic_amplitude,set_language,set_steamvr_lifecycle,open_module_page])
        .build(tauri::generate_context!()).expect("tauri application error")
        .run(|app,event|match event{
            // Destroying the last window (tray mode) must not quit the app; explicit exit passes a code.
            RunEvent::ExitRequested{api,code:None,..}=>api.prevent_exit(),
            RunEvent::Exit=>{
                let state=app.state::<Shared>();
                if state.adb_used.load(Ordering::SeqCst){if let Ok(path)=adb_path(app){let _=adb(&path).arg("kill-server").output();}}
            }
            _=>{}
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample(y:u16,force:f32)->Sensor{Sensor{x:80,y,force}}
    #[test]
    fn button_hysteresis(){
        let idle=[false;2];
        assert_eq!(button_step(idle,&sample(80,0.29),"button",0.30),idle);
        let pressed=button_step(idle,&sample(80,0.30),"button",0.30);
        assert_eq!(pressed,[true,false]);
        assert_eq!(button_step(pressed,&sample(80,0.21),"button",0.30),pressed);
        assert_eq!(button_step(pressed,&sample(80,0.20),"button",0.30),idle);
    }
    #[test]
    fn two_buttons_latch_the_initial_half(){
        let left=button_step([false;2],&sample(127,0.35),"two_buttons",0.30);
        assert_eq!(left,[true,false]);
        assert_eq!(button_step(left,&sample(200,0.35),"two_buttons",0.30),left);
        assert_eq!(button_step([false;2],&sample(128,0.35),"two_buttons",0.30),[false,true]);
        assert_eq!(button_step(left,&Sensor{x:0,y:255,force:0.35},"two_buttons",0.30),[false;2]);
        assert_eq!(button_step(left,&sample(80,0.35),"touchpad",0.30),[false;2]);
    }
    #[test]
    fn config_defaults_missing_fields(){
        let config:UserConfig=serde_json::from_str("{}").unwrap();
        assert_eq!(config.transport,"lan");
        assert_eq!(config.input_mode,"touchpad");
        assert_eq!(config.press_threshold,0.30);
        assert_eq!(config.haptic_amplitude,0.20);
        assert_eq!(config.language,system_language());
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
        let config=UserConfig{transport:"lan".into(),input_mode:"two_buttons".into(),press_threshold:0.45,haptic_amplitude:0.65,language:"ja".into(),steamvr_lifecycle:false};
        write_user_config_file(&file,&config).unwrap();
        let loaded=read_user_config_file(&file).unwrap();
        assert_eq!(loaded.transport,config.transport);
        assert_eq!(loaded.input_mode,config.input_mode);
        assert_eq!(loaded.press_threshold,config.press_threshold);
        assert_eq!(loaded.haptic_amplitude,config.haptic_amplitude);
        assert_eq!(loaded.language,config.language);
        assert!(!loaded.steamvr_lifecycle);
        assert_eq!(std::fs::read_to_string(dir.join("steamvr_lifecycle")).unwrap(),"0");
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn supported_languages_are_validated(){
        for language in ["ru","en","zh-CN","hi","es","ar","fr","bn","pt-BR","id","ja","de"]{assert!(valid_language(language));}
        assert!(!valid_language("xx"));
    }
}
