use crate::app::{RustForgeApp, SidebarTab};
use egui::{Button, Layout, RichText, Ui, Vec2};

pub fn render_activity_bar(app: &mut RustForgeApp, ui: &mut Ui) {
    ui.vertical(|ui| {
        ui.add_space(8.0);

        let make_tab_btn = |ui: &mut Ui, label: &str, tab: SidebarTab, current: SidebarTab, is_visible: bool| {
            let is_selected = is_visible && current == tab;
            let bg_color = if is_selected {
                app.theme.fg_accent
            } else {
                app.theme.bg_activity_bar
            };
            let text_color = if is_selected {
                egui::Color32::WHITE
            } else {
                app.theme.fg_secondary
            };

            let btn = Button::new(RichText::new(label).size(15.0).color(text_color))
                .fill(bg_color)
                .min_size(Vec2::new(42.0, 40.0));

            ui.add(btn)
        };

        // Explorer Button
        if make_tab_btn(ui, "📁", SidebarTab::Explorer, app.active_sidebar, app.sidebar_visible)
            .on_hover_text("Explorer (Ctrl+Shift+E)")
            .clicked()
        {
            if app.sidebar_visible && app.active_sidebar == SidebarTab::Explorer {
                app.sidebar_visible = false;
            } else {
                app.active_sidebar = SidebarTab::Explorer;
                app.sidebar_visible = true;
            }
        }

        ui.add_space(4.0);

        // Search Button
        if make_tab_btn(ui, "🔍", SidebarTab::Search, app.active_sidebar, app.sidebar_visible)
            .on_hover_text("Search (Ctrl+Shift+F)")
            .clicked()
        {
            if app.sidebar_visible && app.active_sidebar == SidebarTab::Search {
                app.sidebar_visible = false;
            } else {
                app.active_sidebar = SidebarTab::Search;
                app.sidebar_visible = true;
            }
        }

        ui.add_space(4.0);

        // Git Button
        if make_tab_btn(ui, "🌿", SidebarTab::Git, app.active_sidebar, app.sidebar_visible)
            .on_hover_text("Source Control (Ctrl+Shift+G)")
            .clicked()
        {
            if app.sidebar_visible && app.active_sidebar == SidebarTab::Git {
                app.sidebar_visible = false;
            } else {
                app.active_sidebar = SidebarTab::Git;
                app.sidebar_visible = true;
            }
        }

        ui.add_space(4.0);

        // Terminal Toggle Button
        let term_btn = Button::new(RichText::new("⌨").size(15.0).color(app.theme.fg_secondary))
            .fill(app.theme.bg_activity_bar)
            .min_size(Vec2::new(42.0, 40.0));
        if ui.add(term_btn).on_hover_text("Toggle Terminal (Ctrl+`)").clicked() {
            app.terminal_visible = !app.terminal_visible;
        }

        ui.with_layout(Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add_space(8.0);

            // Settings Button
            if make_tab_btn(ui, "⚙", SidebarTab::Settings, app.active_sidebar, app.sidebar_visible)
                .on_hover_text("Settings (Ctrl+,)")
                .clicked()
            {
                if app.sidebar_visible && app.active_sidebar == SidebarTab::Settings {
                    app.sidebar_visible = false;
                } else {
                    app.active_sidebar = SidebarTab::Settings;
                    app.sidebar_visible = true;
                }
            }
        });
    });
}
