use ssh2::Session;
use std::io;
use std::io::Read;
use std::net::TcpStream;
use std::process::Command;
extern crate clap;
use clap::{App, Arg};
extern crate chrono;
use chrono::prelude::*;

/*
scripts for remote linux to source:
> cat ~/.remote_cmd.sh

function win()
{
        [ ! -e ~/.remote_in ] && echo "not supprt" && return 1;
        echo "$(pwd)" > ~/.remote_path
        echo "$@" > ~/.remote_in
        cat ~/.remote_out
}

function adb()
{
        win "adb $@"
}

function dakai()
{
        win "start $@"
}
*/

fn main() {
     let matches = App::new("ssh-cmd-export")
          .version("0.1")
          .author("zhuangqiubin<zhuangqiubin@gmail.com>")
          .about("support run local cmd from remote linux")
          .arg(Arg::with_name("local_path")
               .short("l")
               .long("local")
               .value_name("LOCAL_PATH")
               .help(r"local mount point for remote server, like Z:\")
               .takes_value(true))
          .arg(Arg::with_name("remote_path")
               .short("r")
               .long("remote")
               .value_name("REMOTE_PATH")
               .help(r"remote path which mount to local machine, like /home/yourname")
               .takes_value(true))
          .arg(Arg::with_name("server")
               .short("s")
               .long("server")
               .value_name("SERVER")
               .help(r"server, like 192.168.12.34")
               .takes_value(true))
          .arg(Arg::with_name("user")
               .short("u")
               .long("user")
               .value_name("USER")
               .help(r"ssh user")
               .takes_value(true))
          .arg(Arg::with_name("passwd")
               .short("p")
               .long("passwd")
               .value_name("PASSWD")
               .help(r"ssh passwd")
               .takes_value(true))
          .get_matches();

     let mut path_l:String = matches.value_of("local_path").unwrap_or("").into();
     if path_l.len() == 0 {
          println!(r"please enter remote disk in local path:(for example Z:\)");
          io::stdin().read_line(&mut path_l).unwrap();
     }
     let path_l = path_l.trim();
     println!("local path: {}", path_l);

     let mut path_r:String = matches.value_of("remote_path").unwrap_or("").into();
     if path_r.len() == 0 {
          println!(r"please enter remote path:(for example /home/yourname)");
          io::stdin().read_line(&mut path_r).unwrap();
     }
     let path_r = path_r.trim();
     println!("remote path: {}", path_r);

     let mut server:String = matches.value_of("server").unwrap_or("").into();
     if server.len() == 0 {
          println!("please enter server:(for example 192.168.12.34)");
          io::stdin().read_line(&mut server).unwrap();
     }
     let server = server.trim();
     println!("server: {}", server);

     let mut user:String = matches.value_of("user").unwrap_or("").into();
     if user.len() == 0 {
          println!("please enter user:");
          io::stdin().read_line(&mut user).unwrap();
     }
     let user = user.trim();
     println!("user: {}", user);

     let mut passwd:String = matches.value_of("passwd").unwrap_or("").into();
     if passwd.len() == 0 {
          println!("please enter passwd:");
          io::stdin().read_line(&mut passwd).unwrap();
     }
     let passwd = passwd.trim();
     println!("passwd: {}", passwd);

     println!("-------------- connecting {}--------------",
          Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
     let tcp = TcpStream::connect(server.to_string() + (":22")).unwrap();
     let mut sess = Session::new().unwrap();
     sess.set_tcp_stream(tcp);
     sess.handshake().unwrap();

     println!("-------------- authing {}--------------",
          Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
     sess.userauth_password(&user, &passwd).unwrap();
     assert!(sess.authenticated());

     let mut channel = sess.channel_session().unwrap();
     channel.exec("rm -f .remote_in; rm -f .remote_out;rm -f .remote_path;mkfifo .remote_in;mkfifo .remote_out;mkfifo .remote_path").unwrap();
     channel.wait_close().ok();
     println!("-------------- loop {}--------------",
     Local::now().format("%Y-%m-%d %H:%M:%S").to_string());

     loop {
          let mut win_cmd = Command::new("cmd");

          //wait path
          println!("-------------- waiting path {}--------------",
               Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
          let mut remote_path = String::new();
          let mut channel = sess.channel_session().unwrap();
          channel.exec("cat .remote_path").unwrap();
          channel.read_to_string(&mut remote_path).unwrap();
          let remote_path_tmp = str::replace(&remote_path, &path_r, &path_l);
          let remote_path_l = str::replace(&remote_path_tmp, r"/", r"\");
          if remote_path_l.len() == 0 {
               println!("invalid path");
               continue;
          } else {
               print!("path remote: {}", remote_path);
               print!("path local: {}", remote_path_l);
               win_cmd.current_dir(&remote_path_l.trim());
          }
          channel.wait_close().ok();

          //wait cmd
          println!("-------------- waiting cmd {}--------------",
               Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
          let mut remote_cmd = String::new();
          let mut channel = sess.channel_session().unwrap();
          channel.exec("cat .remote_in").unwrap();
          channel.read_to_string(&mut remote_cmd).unwrap();

          let remote_cmd_l = str::replace(&remote_cmd, &path_r, &path_l);
          print!("remote_cmd:{}", remote_cmd);
          print!("remote_cmd:{}", remote_cmd_l);
          channel.wait_close().ok();

          //run cmd
          win_cmd.args(&["/C", &remote_cmd_l]);
          let output = win_cmd.output().expect("failed to execute process");

          //get result
          let remote_ret = String::from_utf8_lossy(&output.stdout);
          println!("ret: {}", remote_ret);
          let result = format!(r#"echo "{}" > .remote_out"#, remote_ret);
          println!("-------------- sending output {}--------------",
               Local::now().format("%Y-%m-%d %H:%M:%S").to_string());

          //return result to remote
          let mut channel = sess.channel_session().unwrap();
          channel.exec(&result).unwrap();
          channel.wait_close().ok();
     }
     //FIXME: unreachable statement
     let mut channel = sess.channel_session().unwrap();
     channel.exec("rm -f .remote_in; rm -f .remote_out; rm -f .remote_path").unwrap();
     channel.wait_close().ok();
     println!("{}", channel.exit_status().unwrap());
}
