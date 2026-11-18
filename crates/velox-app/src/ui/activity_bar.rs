use crate::app::{VeloxApp, SidebarTab};
use crate::ui::icons::Icon;
use egui::{pos2, Color32, Layout, Rect, Sense, Ui, Vec2};

fn render_nav_button(
    ui: &mut Ui,
    icon: Icon,
    is_active: bool,
    tooltip: &str,
    theme: &velox_themes::Theme,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(48.0, 40.0), Sense::click());

    // Hover background highlight
    if response.hovered() {
        ui.painter().rect_filled(
            rect.shrink2(Vec2::new(4.0, 2.0)),
            3.0,
            Color32::from_rgb(26, 27, 35),
        );
    }

    // Active left pill (2px width cyan)
    if is_active {
        let pill_rect = Rect::from_min_max(
            pos2(rect.min.x, rect.min.y + 8.0),
            pos2(rect.min.x + 2.0, rect.max.y - 8.0),
        );
        ui.painter().rect_filled(pill_rect, 1.0, theme.fg_accent);
    }

    let icon_color = if is_active {
        theme.fg_primary
    } else if response.hovered() {
        theme.fg_secondary
    } else {
        theme.fg_muted
    };

    let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(18.0));
    icon.paint(ui.painter(), icon_rect, icon_color);
    response.on_hover_text(tooltip)
}

pub fn render_activity_bar(app: &mut VeloxApp, ui: &mut Ui) {
    ui.vertical(|ui| {
        ui.add_space(8.0);

        // Explorer Button
        let explorer_active = app.sidebar_visible && app.active_sidebar == SidebarTab::Explorer;
        if render_nav_button(
            ui,
            Icon::Files,
            explorer_active,
            "Explorer (Ctrl+Shift+E)",
            &app.theme,
        )
        .clicked()
        {
            if explorer_active {
                app.sidebar_visible = false;
            } else {
                app.active_sidebar = SidebarTab::Explorer;
                app.sidebar_visible = true;
            }
        }

        ui.add_space(4.0);

        // Search Button
        let search_active = app.sidebar_visible && app.active_sidebar == SidebarTab::Search;
        if render_nav_button(
            ui,
            Icon::Search,
            search_active,
            "Search (Ctrl+Shift+F)",
            &app.theme,
        )
        .clicked()
        {
            if search_active {
                app.sidebar_visible = false;
            } else {
                app.active_sidebar = SidebarTab::Search;
                app.sidebar_visible = true;
            }
        }

        ui.add_space(4.0);

        // Git Button
        let git_active = app.sidebar_visible && app.active_sidebar == SidebarTab::Git;
        if render_nav_button(
            ui,
            Icon::SourceControl,
            git_active,
            "Source Control (Ctrl+Shift+G)",
            &app.theme,
        )
        .clicked()
        {
            if git_active {
                app.sidebar_visible = false;
            } else {
                app.active_sidebar = SidebarTab::Git;
                app.sidebar_visible = true;
            }
        }

        ui.add_space(4.0);

        // Terminal Toggle Button
        if render_nav_button(
            ui,
            Icon::Terminal,
            app.terminal_visible,
            "Toggle Terminal (Ctrl+`)",
            &app.theme,
        )
        .clicked()
        {
            app.terminal_visible = !app.terminal_visible;
        }

        ui.with_layout(Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add_space(8.0);

            // Settings Button
            let settings_active = app.sidebar_visible && app.active_sidebar == SidebarTab::Settings;
            if render_nav_button(
                ui,
                Icon::Settings,
                settings_active,
                "Settings (Ctrl+,)",
                &app.theme,
            )
            .clicked()
            {
                if settings_active {
                    app.sidebar_visible = false;
                } else {
                    app.active_sidebar = SidebarTab::Settings;
                    app.sidebar_visible = true;
                }
            }
        });
    });
}
