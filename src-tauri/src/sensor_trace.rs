//! Optional capture of hardware values before normalization/clamping.
//! Set QPTP_SENSOR_TRACE to a CSV path before starting the application.
use std::{fs::{File,OpenOptions},io::Write,sync::{Mutex,OnceLock}};
use crate::Sensor;

pub fn sample(time_us:u64,left:&Sensor,right:&Sensor) {
    static FILE:OnceLock<Option<Mutex<File>>>=OnceLock::new();
    let Some(file)=FILE.get_or_init(||{
        let path=std::env::var_os("QPTP_SENSOR_TRACE")?;
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file)=>{
                if writeln!(file,"time_us,side,x,y,force,size,contact,proximity,slide").is_err(){return None}
                Some(Mutex::new(file))
            },
            Err(e)=>{log::warn!("Cannot create sensor trace {:?}: {e}",path);None}
        }
    }) else{return};
    if let Ok(mut file)=file.lock(){
        for (side,s) in [("left",left),("right",right)] {
            let _=writeln!(file,"{time_us},{side},{},{},{},{},{},{},{}",s.x,s.y,s.force,s.size.unwrap_or(0.0),u8::from(s.touch==Some(true)),s.trigger_proximity,s.trigger_slide);
        }
        let _=file.flush();
    }
}
