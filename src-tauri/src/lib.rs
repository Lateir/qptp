use serde::Serialize;
use serde_json::Value;
use std::{io::{self, Read}, net::{TcpStream, UdpSocket}, path::PathBuf, process::Command, sync::{atomic::{AtomicBool, AtomicU64, Ordering}, Arc, Mutex}, thread, time::{Duration, Instant}};
use tauri::{image::Image, menu::{IconMenuItem, Menu, MenuItem, PredefinedMenuItem}, tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent}, AppHandle, Emitter, Manager, RunEvent, State, WindowEvent, Wry};

const PORT: u16 = 27182;
const DISCOVERY_PORT: u16 = 27183;

#[derive(Clone, Default, Serialize)]
struct Sensor { x: u16, y: u16, force: f32 }
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    phase: String, message: String, transport: String, endpoint: Option<String>,
    protocol: Option<String>, samples: u64, left: Sensor, right: Sensor, status: Option<Value>,
}
impl Default for Snapshot {
    fn default() -> Self { Self { phase:"searching".into(), message:"Поиск устройства…".into(), transport:"usb".into(), endpoint:None, protocol:None, samples:0, left:Sensor::default(), right:Sensor::default(), status:None } }
}
struct StreamState { snapshot: Mutex<Snapshot>, generation: AtomicU64, adb_used: AtomicBool }
type Shared = Arc<StreamState>;
struct Tray { left: IconMenuItem<Wry>, right: IconMenuItem<Wry>, last: Mutex<Option<(bool,bool)>> }

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
    let lr=match state.snapshot.lock() {Ok(mut snapshot)=>{change(&mut snapshot);let _=app.emit("stream-state",snapshot.clone());(controller_connected(&snapshot,"left"),controller_connected(&snapshot,"right"))},Err(_)=>return};
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
fn read_frame(stream:&mut TcpStream, app:&AppHandle, state:&Shared, generation:u64)->io::Result<()>{
    let mut header=[0u8;4];let mut last_emit=Instant::now()-Duration::from_secs(1);
    loop {
        if state.generation.load(Ordering::SeqCst)!=generation{return Ok(())}
        read_exact_checked(stream,&mut header,state,generation)?;
        match &header {
            b"QPS1"=>{
                let mut size=[0u8;4];read_exact_checked(stream,&mut size,state,generation)?;
                let size=u32::from_le_bytes(size) as usize;
                if size>4096{return Err(io::Error::new(io::ErrorKind::InvalidData,"Слишком большой статус"))}
                let mut body=vec![0u8;size];read_exact_checked(stream,&mut body,state,generation)?;
                let status:Value=serde_json::from_slice(&body).map_err(io::Error::other)?;
                if state.generation.load(Ordering::SeqCst)!=generation {return Ok(())}
                publish(app,state,|s|{s.status=Some(status);s.phase="connected".into();s.message="Поток сенсоров активен".into();s.protocol=Some("QPR2 / QPS1".into())});
            }
            b"QPR2"=>{
                let mut body=[0u8;28];read_exact_checked(stream,&mut body,state,generation)?;
                if state.generation.load(Ordering::SeqCst)!=generation {return Ok(())}
                let (left,right)=(sensor(&body[12..20]),sensor(&body[20..28]));
                if let Ok(mut s)=state.snapshot.lock(){s.left=left;s.right=right;s.samples+=1;s.phase="connected".into();s.message="Поток сенсоров активен".into();s.protocol=Some("QPR2 / QPS1".into());if last_emit.elapsed()>=Duration::from_millis(33){let _=app.emit("stream-state",s.clone());last_emit=Instant::now()}}
            }
            _=>return Err(io::Error::new(io::ErrorKind::InvalidData,"Неизвестный формат потока"))
        }
    }
}
fn worker(app:AppHandle,state:Shared,generation:u64,transport:String){
    let mut owned_forward:Option<PathBuf>=None;
    loop {
        if state.generation.load(Ordering::SeqCst)!=generation {break}
        publish(&app,&state,|s|{s.phase="searching".into();s.message=if transport=="usb"{"Поиск Quest через ADB…".into()}else{"Поиск Quest в локальной сети…".into()};s.status=None;s.endpoint=None;s.protocol=None;s.left=Sensor::default();s.right=Sensor::default()});
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
fn start_worker(app:&AppHandle,state:&Shared,transport:String){
    let generation=state.generation.fetch_add(1,Ordering::SeqCst)+1;
    publish(app,state,|s|{*s=Snapshot{transport:transport.clone(),..Snapshot::default()}});
    let (app,shared)=(app.clone(),state.clone());thread::spawn(move||worker(app,shared,generation,transport));
}
#[tauri::command]
fn get_stream_state(state:State<Shared>)->Snapshot{state.snapshot.lock().unwrap().clone()}
#[tauri::command]
fn set_transport(app:AppHandle,state:State<Shared>,transport:String)->Result<(),String>{
    if transport!="usb"&&transport!="lan"{return Err("Неизвестный способ подключения".into())}
    if state.snapshot.lock().map(|s|s.transport==transport).unwrap_or(false){return Ok(())}
    if let Some(f)=transport_file(&app){let _=f.parent().map(std::fs::create_dir_all);let _=std::fs::write(f,&transport);}
    start_worker(&app,state.inner(),transport);Ok(())
}
fn show_window(app:&AppHandle){
    if let Some(w)=app.get_webview_window("main"){let _=w.unminimize();let _=w.show();let _=w.set_focus();return}
    // Building a webview inside an event handler can deadlock on Windows, so do it off the main thread.
    let app=app.clone();
    thread::spawn(move||{if let Some(cfg)=app.config().app.windows.first(){if let Ok(w)=tauri::WebviewWindowBuilder::from_config(&app,cfg).and_then(|b|b.build()){let _=w.set_focus();}}});
}
fn setup_tray(app:&AppHandle)->tauri::Result<()>{
    let left=IconMenuItem::with_id(app,"left","L",true,Some(dot(false)),None::<&str>)?;
    let right=IconMenuItem::with_id(app,"right","R",true,Some(dot(false)),None::<&str>)?;
    let menu=Menu::with_items(app,&[&left,&right,&PredefinedMenuItem::separator(app)?,&MenuItem::with_id(app,"show","Открыть",true,None::<&str>)?,&MenuItem::with_id(app,"quit","Выход",true,None::<&str>)?])?;
    let mut tray=TrayIconBuilder::with_id("main").tooltip("Quest Pro Touch Plus").menu(&menu).show_menu_on_left_click(false)
        .on_menu_event(|app,e|match e.id().as_ref(){"show"=>show_window(app),"quit"=>app.exit(0),_=>{}})
        .on_tray_icon_event(|tray,e|if let TrayIconEvent::Click{button:MouseButton::Left,button_state:MouseButtonState::Up,..}=e{show_window(tray.app_handle())});
    if let Some(icon)=app.default_window_icon(){tray=tray.icon(icon.clone())}
    tray.build(app)?;
    app.manage(Tray{left,right,last:Mutex::new(None)});
    Ok(())
}
pub fn run(){
    let state:Shared=Arc::new(StreamState{snapshot:Mutex::new(Snapshot::default()),generation:AtomicU64::new(0),adb_used:AtomicBool::new(false)});
    tauri::Builder::default().manage(state)
        .setup(|app|{
            let handle=app.handle().clone();
            setup_tray(&handle)?;
            let transport=transport_file(&handle).and_then(|f|std::fs::read_to_string(f).ok()).map(|t|t.trim().to_owned()).filter(|t|t=="usb"||t=="lan").unwrap_or_else(||"usb".into());
            start_worker(&handle,handle.state::<Shared>().inner(),transport);
            Ok(())
        })
        .on_window_event(|window,event|match event{
            // Minimize = hide to tray: destroy the window so WebView2 releases its memory.
            WindowEvent::Resized(_) if window.is_minimized().unwrap_or(false)=>{let _=window.destroy();}
            WindowEvent::CloseRequested{..}=>window.app_handle().exit(0),
            _=>{}
        })
        .invoke_handler(tauri::generate_handler![get_stream_state,set_transport])
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
