// #![windows_subsystem = "windows"]
#[macro_use]
extern crate sciter;
// use libloading::{Symbol, Library};
// use libc::*;
// use std::{ffi::CString, ptr::null_mut};
// use sciter::value;
use std::fs;
use crossbeam_utils::thread;
use crossbeam_queue::ArrayQueue;

struct Handler<'a>{
    q:&'a ArrayQueue<String>,
    dist: String
}

impl<'a> Handler<'a>{
    fn index_sum(&self, lx:i32, sd:String, ed:String) -> String {
        let msg = format!("+;{};{};{}",lx,sd,ed);
        &self.q.push(msg);
        let mut res = String::new();
        loop{
            if self.q.is_full() {
                if let Some(c) = self.q.pop(){
                    if c.starts_with("+") {
                        if self.q.is_empty(){
                            &self.q.push(c);
                        }
                    }else{
                        res = c;
                        break;
                    }
                };
            }
        }
        res
    }
    fn write(&self, d:String,e:String,f:String) -> i32 {
        0
    }
    fn exporto(&self, o:i32) {
        if o == 0 {
            fs::copy("./temp.csv", format!("{}/挂接查询.csv",&self.dist)).unwrap();
        }else if o == 1 {
            fs::copy("./record.csv", format!("{}/人工记录.csv",&self.dist)).unwrap();
        }
    }
}
impl<'a> sciter::EventHandler for Handler<'a> {
    dispatch_script_call! {
        fn index_sum(i32, String, String);
        fn exporto(i32);
        fn write(String,String,String);
    }
}

#[async_std::main]
async fn main() -> anyhow::Result<()>{
    use once_cell::sync::Lazy;
    use tiberius::{AuthMethod, Client, Config};
    use async_std::net::TcpStream;
    static CONN_STR: Lazy<String> = Lazy::new(|| {
        std::env::var("TIBERIUS_CONNECTION_STRING").unwrap_or_else(|_| {
            "server=tcp:localhost,1433;IntegratedSecurity=true;TrustServerCertificate=true".to_owned()
        })
    });
    let mut config = Config::from_ado_string(&CONN_STR)?;
    config.database("cb");
    config.authentication(AuthMethod::sql_server("sa", "12345"));
    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;
    let mut client = Client::connect(config, tcp).await?;

    let q = ArrayQueue::new(1);
    let dist: String = std::env::args().nth(1).unwrap();
    thread::scope(|sc|{
        sc.spawn(|_| {
            let html = include_bytes!("../ui/clock.htm");
            // let resouces = include_bytes!("../ui.rc");
            let mut frame = sciter::WindowBuilder::main_window().with_size((400, 300)).with_pos((540,270)).create();
            let load = format!("file://{}/ui/clock.htm", std::env::current_dir().unwrap().to_str().unwrap());
            frame.load_html(html, Some(&load));
            // frame.archive_handler(resouces).expect("Invalid archive");
            let handler = Handler{q:&q,dist:dist};
            frame.event_handler(handler);
            frame.load_file("this://app/clock.htm");
            
            frame.run_app();
        });
        loop {
            if q.is_full() {
                if let Some(c) = q.pop(){
                    if c.starts_with("+") {
                        let args:Vec<&str> = c.splitn(4,";").collect();
                        dbg!(&args);
                        let que = format!("SELECT PID,DID,TITLE from dbo.E_FILE{} WHERE STATUS = 0 AND CREATETIME BETWEEN '{}' AND '{}'", args[1],args[2],args[3]);
                        let stream = client
                            .query(que,
                                &[&1, &2, &3],
                            )
                            .await?;
                        let rowsets = stream.into_results().await?;
                        let mut ts = 0;
                        let mut zys = 0;
                        let mut nys = 0;
                        let mut ys0 = 0;
                        if let Some(rows) = rowsets.get(0) {
                            for row in rows {
                                ts = ts+1;
                                let pid =row.get::<i32, _>(0).unwrap();
                                let u = format!("SELECT YS,TITLE from dbo.D_FILE{} WHERE STATUS = 0 AND DID = {}", args[1], pid);
                                let resu = client
                                    .query(u, 
                                        &[&1, &2]
                                    )
                                    .await?;
                                    let res = resu.into_results().await?;
                                    if let Some(re) = res.get(0){
                                        for r in re {
                                            let ys = r.get::<i32, _>(0);
                                            if ys != None {
                                                let s = ys.unwrap();
                                                if s == 0 {
                                                    ys0 = ys0+1;
                                                }else {
                                                    zys = zys + s;
                                                }
                                            }else{
                                                nys = nys+1;
                                            }
                                        }
                                    }
                            }
                        }
                        let msg = format!("-{} - {}，挂接数：{}件，总页数：{}页，页数为空：{}条，页数为0：{}条", args[2],args[3],ts, zys, nys, ys0);
                        dbg!(&msg);
                        if q.is_empty(){
                            q.push(msg).unwrap();
                        }
                    }else{
                        if q.is_empty(){
                            q.push(c).unwrap();
                        }
                    }
                };
            }
        }
    }).unwrap();
    
    Ok(())
}

// fn _token() -> bool {
//     const LIB: &'static str = "token.dll";
//     type Find=unsafe fn(*const c_uchar, *mut c_int) -> i32;
//     type Open=unsafe fn(*const *mut c_void, *const c_uchar, c_int) -> i32;
//     type Verify=unsafe fn(*const c_void, c_int, *const c_uchar) -> i32;
//     type Close=unsafe fn(*const c_void) -> i32;
//     unsafe {
//         let lib = Library::new(LIB).expect("请保证系统文件完整！");
//         let l = std::fs::read(LIB).unwrap();
//         if l.len() != 87552 {
//             return false;
//         }
//         let et_find: Symbol<Find> = lib.get(b"et_FindToken").unwrap();
//         let mut count = 0;
//         let id = CString::new("64761549").unwrap();
//         et_find(id.as_bytes().as_ptr(), &mut count);
//         if count>0 {
//             let handle: *mut c_void = null_mut();
//             let et_open: Symbol<Open> = lib.get(b"et_OpenToken").unwrap();
//             if et_open(&handle, id.as_bytes().as_ptr(), 1) == 0 {
//                 let et_verify: Symbol<Verify> = lib.get(b"et_Verify").unwrap();
//                 let word = CString::new("19940501EDFABCAB").unwrap();
//                 let v = et_verify(handle as *const c_void, 0, word.as_bytes().as_ptr());
//                 let et_close: Symbol<Close> = lib.get(b"et_CloseToken").unwrap();
//                 et_close(handle);
//                 if v == 0{
//                     true
//                 }else{
//                     false
//                 }
//             }else{
//                 false
//             }
//         }else{
//             false
//         }
//     }
// }