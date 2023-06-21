use std::{path::PathBuf, str::FromStr};

use install::check_modloaders;
use native_dialog::FileDialog;
use slint::SharedString;

mod find_gd;
mod install;
mod extension;

slint::slint! {
    import { StandardButton} from "std-widgets.slint";

    export component ModLoaderAlert inherits Dialog {
        title: "Notice";
        in property <string> info;
        Text {
            text: info;
        }
        StandardButton { kind: ok; }
        StandardButton { kind: cancel; }
    }
}

slint::slint! {
    import { Button, ComboBox, CheckBox, LineEdit } from "std-widgets.slint";

    component GDPathInput {
        pure callback get-value() -> string;
        pure callback pick-dialog() -> string;
        callback set-value(string);

        VerticalLayout {
            Text {
                text: "Enter the Path to GD...";
            }
            HorizontalLayout {
                input := LineEdit {
                    placeholder-text: "Path to Geometry Dash";
                    text: root.get-value();
                    edited(value) => {
                        root.set-value(value);
                    }
                }
                Button {
                    text: "...";
                    clicked => {
                        input.text = root.pick-dialog();
                        root.set-value(input.text);
                    }
                }
            }
        }
    }

    component ProgressBar {
        in property <percent> progress;

        Rectangle {
            x: 0;
            y: 0;
            width: 100%;
            height: 15px;
            background: #aaa;
        }

        Rectangle {
            x: 0;
            y: 0;
            width: progress;
            height: 15px;
            background: #72f872;
        }

        Rectangle {
            x: 0;
            y: 0;
            width: 100%;
            height: 15px;
            border-color: #555;
            border-width: 1px;
        }
    }

    export component MainWindow inherits Window {
        title: "Geode Installer";
        icon: @image-url("assets/icon.png");
        min-width: 320px;
        min-height: 320px;
        preferred-width: 380px;
        preferred-height: 320px;
        background: #eee;

        out property <string> gd-path: find-gd-path();
        property <bool> manual-path-enter: false;
        property <int> path-is-valid: root.validate-path(root.gd-path);
        property <int> installing: 0;
        in property <percent> install-progress: 0%;

        pure callback find-gd-path() -> string;
        pure callback validate-path(string) -> int;
        pure callback pick-path() -> string;
        pure callback platform-install-info() -> string;

        pure callback begin-install();
        pure callback begin-uninstall();
        pure callback cancel-install();

        public function load_initial_status() {
            if (path-is-valid == 0) {
                root.manual-path-enter = true;
            }
        }

        VerticalLayout {
            alignment: space-around;
            padding: 20px;
            VerticalLayout {
                Image {
                    source: @image-url("assets/geode-logo-circle.svg");
                    max-height: 65px;
                }
                Text {
                    text: "Install Geode";
                    font-size: 20px;
                    font-weight: 500;
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }
            }
            if !manual-path-enter && installing == 0 : VerticalLayout {
                alignment: center;
                spacing: 5px;
                Text {
                    text: "Installing to Steam GD";
                    horizontal-alignment: center;
                }
                HorizontalLayout {
                    alignment: center;
                    Button {
                        text: "Choose another location";
                        clicked => {
                            root.manual-path-enter = true;
                        }
                    }
                }
            }
            if manual-path-enter && installing == 0 : GDPathInput {
                get-value => {
                    return root.gd-path;
                }
                set-value(value) => {
                    root.gd-path = value;
                }
                pick-dialog() => {
                    return root.pick-path();
                }
            }
            if installing != 0 : VerticalLayout {
                spacing: 5px;
                Text {
                    text: root.installing == 1 ? "Installing..." : "Uninstalling...";
                }
                ProgressBar {
                    progress: root.install-progress;
                }
            }
            if installing == 0 : Text {
                wrap: word-wrap;
                horizontal-alignment: center;
                color: path-is-valid == 0 ? #9c1515 : #2c832c;
                text: {
                    if (path-is-valid == 1) {
                        "Ready to install"
                    }
                    else if (path-is-valid == 2) {
                        "Geode is already installed in the selected location! " + 
                            "Would you like to reinstall or uninstall it?"
                    }
                    else {
                        "This does not seem like a valid installation of Geometry Dash 2.113! " +
                            root.platform-install-info()
                    }
                }
            }
            HorizontalLayout {
                alignment: center;
                if installing != 0 : Button {
                    text: "Cancel";
                    clicked => {
                        root.installing = 0;
                        cancel-install();
                    }
                }
                if installing == 0 : Button {
                    text: path-is-valid == 2 ? "Reinstall" : "Install";
                    enabled: path-is-valid != 0;
                    clicked => {
                        root.installing = 1;
                        begin-install();
                    }
                }
                if path-is-valid == 2 && installing == 0 : Button {
                    text: "Uninstall";
                    clicked => {
                        root.installing = 2;
                        begin-uninstall();
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let win = MainWindow::new()?;
    win.on_find_gd_path(move || {
        find_gd::find_gd_path().unwrap_or(String::new()).into()
    });
    win.on_validate_path(move |path: SharedString| {
        match PathBuf::from_str(&path)
            .ok()
            .and_then(|p| Some(install::check_for_gd(&p)))
            .unwrap_or(install::FoundGD::None)
        {
            install::FoundGD::None => 0,
            install::FoundGD::Vanilla(_) => 1,
            install::FoundGD::Geode(_) => 2,
        }
    });
    let handle = win.as_weak();
    win.on_pick_path(move || {
        let ui = handle.unwrap();
        let mut dialog = FileDialog::new();
        let path = PathBuf::from_str(&ui.get_gd_path())
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));
        if let Some(ref path) = path {
            dialog = dialog.set_location(path);
        }
        if cfg!(target_os = "macos") {
            dialog = dialog.add_filter("Application", &["app"]);
        } else if cfg!(windows) {
            dialog = dialog.add_filter("Executable", &["exe"]);
        }
        dialog.show_open_single_file().ok().flatten()
            .map(|p| SharedString::from(p.to_string_lossy().as_ref()))
            .unwrap_or(ui.get_gd_path())
    });
    win.on_platform_install_info(move || {
        if cfg!(windows) {
            "Make sure the path leads to the Geometry Dash executable (GeometryDash.exe, or the equivalent for your GDPS)"
        }
        else if cfg!(target_os = "macos") {
            ""
        }
        else {
            ""
        }.into()
    });
    let handle = win.as_weak();
    win.on_begin_install(move || {
        let ui = handle.unwrap();
        let path = PathBuf::from_str(&ui.get_gd_path()).unwrap();
        if let Some(info) = check_modloaders(&path) {
            ModLoaderAlert::new().unwrap().show().unwrap();
        }
    });
    win.invoke_load_initial_status();
    win.run()
}
