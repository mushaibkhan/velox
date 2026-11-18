use crate::app::VeloxApp;
use egui::{menu, Ui};
use velox_core::Action;

pub fn render_menu_bar(app: &mut VeloxApp, ui: &mut Ui) {
    menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.add(egui::Button::new("New File").shortcut_text("Ctrl+N")).clicked() {
                app.dispatch_action(Action::NewFile);
                ui.close_menu();
            }
            if ui.add(egui::Button::new("Open File...").shortcut_text("Ctrl+O")).clicked() {
                app.dispatch_action(Action::OpenFile);
                ui.close_menu();
            }
            if ui.button("Open Folder...").clicked() {
                app.open_folder_dialog();
                ui.close_menu();
            }
            ui.separator();
            if ui.add(egui::Button::new("Save").shortcut_text("Ctrl+S")).clicked() {
                app.dispatch_action(Action::SaveFile);
                ui.close_menu();
            }
            if ui.add(egui::Button::new("Save As...").shortcut_text("Ctrl+Shift+S")).clicked() {
                app.dispatch_action(Action::SaveFileAs);
                ui.close_menu();
            }
            ui.separator();
            if ui.add(egui::Button::new("Close Tab").shortcut_text("Ctrl+W")).clicked() {
                app.dispatch_action(Action::CloseTab);
                ui.close_menu();
            }
            ui.separator();
            if ui.add(egui::Button::new("Exit").shortcut_text("Ctrl+Q")).clicked() {
                app.dispatch_action(Action::Quit);
                ui.close_menu();
            }
        });

        ui.menu_button("Edit", |ui| {
            if ui.add(egui::Button::new("Undo").shortcut_text("Ctrl+Z")).clicked() {
                app.dispatch_action(Action::Undo);
                ui.close_menu();
            }
            if ui.add(egui::Button::new("Redo").shortcut_text("Ctrl+Y")).clicked() {
                app.dispatch_action(Action::Redo);
                ui.close_menu();
            }
            ui.separator();
            if ui.add(egui::Button::new("Find").shortcut_text("Ctrl+F")).clicked() {
                app.dispatch_action(Action::Find);
                ui.close_menu();
            }
            if ui.add(egui::Button::new("Replace").shortcut_text("Ctrl+H")).clicked() {
                app.dispatch_action(Action::Replace);
                ui.close_menu();
            }
            ui.separator();
            if ui.add(egui::Button::new("Select All").shortcut_text("Ctrl+A")).clicked() {
                app.dispatch_action(Action::SelectAll);
                ui.close_menu();
            }
        });

        ui.menu_button("View", |ui| {
            if ui.add(egui::Button::new("Command Palette...").shortcut_text("Ctrl+Shift+P")).clicked() {
                app.dispatch_action(Action::CommandPalette);
                ui.close_menu();
            }
            if ui.add(egui::Button::new("Quick Open...").shortcut_text("Ctrl+P")).clicked() {
                app.dispatch_action(Action::QuickOpen);
                ui.close_menu();
            }
            ui.separator();
            if ui.add(egui::Button::new("Toggle Sidebar").shortcut_text("Ctrl+B")).clicked() {
                app.dispatch_action(Action::ToggleSidebar);
                ui.close_menu();
            }
            if ui.add(egui::Button::new("Toggle Terminal").shortcut_text("Ctrl+`")).clicked() {
                app.dispatch_action(Action::ToggleTerminal);
                ui.close_menu();
            }
            ui.separator();
            ui.menu_button("Theme", |ui| {
                if ui.button("Dark Modern").clicked() {
                    app.set_theme(velox_themes::ThemeMode::DarkModern);
                    ui.close_menu();
                }
                if ui.button("Tokyo Night").clicked() {
                    app.set_theme(velox_themes::ThemeMode::TokyoNight);
                    ui.close_menu();
                }
                if ui.button("Monokai Pro").clicked() {
                    app.set_theme(velox_themes::ThemeMode::MonokaiPro);
                    ui.close_menu();
                }
            });
        });

        ui.menu_button("Terminal", |ui| {
            if ui.button("New Terminal").clicked() {
                let ctx = ui.ctx().clone();
                app.new_terminal_session(Some(ctx));
                app.terminal_visible = true;
                ui.close_menu();
            }
            if ui.button("Clear Terminal").clicked() {
                if let Some(term) = app.terminal_mut() {
                    term.grid.write().clear_screen();
                }
                ui.close_menu();
            }
        });

        ui.menu_button("Help", |ui| {
            if ui.button("About Velox").clicked() {
                ui.close_menu();
            }
        });
    });
}
