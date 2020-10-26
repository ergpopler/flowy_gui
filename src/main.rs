use gio::prelude::*;
use gtk::{prelude::*, Application};
use std::{env, io::prelude::*};
use std::sync::mpsc::{TryRecvError, channel};

fn main() {
    let app = Application::new(
        Some("com.flowy.flowy_gui"),
        gio::ApplicationFlags::FLAGS_NONE,
    )
    .expect("GTK initialization failed.");

    app.connect_activate(|app| {
        let glade_src = include_str!("../layout/Layout.glade");
        let browse_src = include_str!("../layout/Browse.glade");
        let builder = gtk::Builder::from_string(glade_src);
        let browse_builder = gtk::Builder::from_string(browse_src);
        let window: gtk::Window = builder.get_object("ApplicationWindow1").unwrap();
        window.set_application(Some(app));
        let browse_window: gtk::FileChooserDialog = browse_builder.get_object("BrowseWindow").unwrap();
        let solar_button: gtk::CheckButton = builder.get_object("SolarButton").unwrap();
        let latitude_input: gtk::Entry = builder.get_object("LatitudeInput").unwrap();
        let longitude_input: gtk::Entry = builder.get_object("LongitudeInput").unwrap();
        let directory_input: gtk::Entry = builder.get_object("DirectoryInput").unwrap();
        let button: gtk::Button = builder.get_object("DoneButton").unwrap();
        let browse_button: gtk::Button = builder.get_object("BrowseButton").unwrap();
        let browse_done_button: gtk::Button = browse_builder.get_object("DoneButton").unwrap();
        let (sender, receiver) = channel();

		browse_button.connect_clicked(move |_|{
            let browse_window_c: gtk::FileChooserDialog = browse_window.clone();
            browse_window_c.show_all();
            let sender_clone = sender.clone();

            browse_done_button.connect_clicked(move |_|{
                let folderpath = browse_window_c.get_current_folder();
                sender_clone.send(folderpath).unwrap();
                browse_window_c.close();
            
            });
        });

        button.connect_clicked(move |_|{
            let is_solar = solar_button.get_active();
            let directory = directory_input.get_text();
            let latitude = latitude_input.get_text();
            let longitude = longitude_input.get_text();
            let desktop_env = String::from_utf8(std::process::Command::new("sh")
                .arg("-c")
                .arg("echo $XDG_CURRENT_DESKTOP")
                .output()
                .expect("Failed to execute")
                .stdout);
                let desktop_env = match desktop_env {
                    Ok(de) => de,
                    Err(error) => panic!("Problem finding desktop environment! {:?}", error),
                }.trim().to_owned();

            make(is_solar, latitude.to_string().parse::<f64>().expect("Please enter a latitude, if you are not using solar than input -1."), longitude.to_string().parse::<f64>().expect("Please enter a longitude, if you are not using solar than input -1."), directory.to_string(), desktop_env).expect("Failed to make service file.");
            println!("put flowy.service into /etc/systemd/user/ then run systemctl --user start flowy.service");
            let pathdir = receiver.try_recv();
            let pathdir = match pathdir{
                Ok(val) => val,
                Err(TryRecvError::Empty) => None,
                Err(err) => panic!("Error receiving from sender..."),
            };
            let pathdir = match pathdir{
                Some(val) => val,
                None => std::path::PathBuf::new(),
              
            };
            
        });

        window.show_all();
    });
    
    app.run(&env::args().collect::<Vec<_>>());
}

fn make(sol: bool, lat: f64, long: f64, dir: String, de: String) -> std::io::Result<()> {
    let service = if sol {
        format!(
            "
[Unit]
Description=flowy

[Service]
Environment=XDG_CURRENT_DESKTOP={}
ExecStart=sh -c 'flowy --solar {} {} {}'
        
[Install]
WantedBy=multi-user.target
",
            de, dir, lat, long
        )
    } else {
        format!(
            "
[Unit]
Description=flowy
        
[Service]
Environment=XDG_CURRENT_DESKTOP={}
ExecStart=sh -c 'flowy --dir {}'
        
[Install]
WantedBy=multi-user.target
",
            de, dir
        )
    };

    let mut file = std::fs::File::create("flowy.service")?;
    file.write_all(format!("{}", service.to_string()).as_bytes())?;
    Ok(())
}

