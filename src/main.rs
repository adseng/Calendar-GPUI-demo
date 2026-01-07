#![windows_subsystem = "windows"]

use gpui::*;
use chrono::{Datelike, NaiveDate, Local};

struct Calendar {
    current_month: NaiveDate,
    selected_date: Option<NaiveDate>,
}

struct DatePicker {
    calendar: Calendar,
    is_open: bool,
    show_above: bool,
}

struct AppView {
    pickers: Vec<DatePicker>,
}

impl Calendar {
    fn new() -> Self {
        let today = Local::now().date_naive();
        Self {
            current_month: today,
            selected_date: Some(today),
        }
    }

    fn prev_month(&mut self) {
        self.current_month = self.current_month
            .with_day(1)
            .unwrap()
            .pred_opt()
            .unwrap()
            .with_day(1)
            .unwrap();
    }

    fn next_month(&mut self) {
        let first_day = self.current_month.with_day(1).unwrap();
        self.current_month = first_day
            .with_month(first_day.month() + 1)
            .unwrap_or_else(|| first_day.with_year(first_day.year() + 1).unwrap().with_month(1).unwrap());
    }

    fn prev_year(&mut self) {
        self.current_month = self.current_month
            .with_year(self.current_month.year() - 1)
            .unwrap();
    }

    fn next_year(&mut self) {
        self.current_month = self.current_month
            .with_year(self.current_month.year() + 1)
            .unwrap();
    }

    fn get_month_days(&self) -> Vec<Option<NaiveDate>> {
        let first_day = self.current_month.with_day(1).unwrap();
        let first_weekday = first_day.weekday().num_days_from_sunday() as usize;
        
        let mut days = Vec::new();
        if first_weekday > 0 {
            let prev_month = first_day.pred_opt().unwrap();
            let days_in_prev_month = prev_month.day();
            for i in (days_in_prev_month - first_weekday as u32 + 1)..=days_in_prev_month {
                days.push(Some(prev_month.with_day(i).unwrap()));
            }
        }

        let days_in_month = first_day
            .with_month(first_day.month() + 1)
            .unwrap_or_else(|| first_day.with_year(first_day.year() + 1).unwrap().with_month(1).unwrap())
            .pred_opt()
            .unwrap()
            .day();
        
        for day in 1..=days_in_month {
            days.push(Some(first_day.with_day(day).unwrap()));
        }

        while days.len() < 42 {
            let last_date = days.last().unwrap().unwrap();
            days.push(last_date.succ_opt());
        }

        days
    }

    fn is_today(&self, date: &NaiveDate) -> bool {
        let today = Local::now().date_naive();
        date.year() == today.year() && date.month() == today.month() && date.day() == today.day()
    }

    fn is_selected(&self, date: &NaiveDate) -> bool {
        if let Some(selected) = self.selected_date {
            date.year() == selected.year() && date.month() == selected.month() && date.day() == selected.day()
        } else {
            false
        }
    }

    fn is_current_month(&self, date: &NaiveDate) -> bool {
        date.year() == self.current_month.year() && date.month() == self.current_month.month()
    }
}

impl DatePicker {
    fn new() -> Self {
        Self {
            calendar: Calendar::new(),
            is_open: false,
            show_above: false,
        }
    }

    fn format_date(&self) -> String {
        if let Some(date) = self.calendar.selected_date {
            format!("{}-{:02}-{:02}", date.year(), date.month(), date.day())
        } else {
            "选择日期".to_string()
        }
    }

    #[allow(dead_code)]
    fn calculate_position(&mut self, window: &Window) {
        let window_bounds = window.inner_window_bounds();
        let bounds = window_bounds.get_bounds();
        let window_height: f32 = bounds.size.height.into();
        
        let input_y = window_height / 2.0;
        let input_height = 40.0;
        let input_bottom = input_y + input_height / 2.0;
        let calendar_height = 320.0;
        let margin = 8.0;
        
        let space_below = window_height - input_bottom;
        let space_above = input_y - input_height / 2.0;
        self.show_above = space_below < calendar_height + margin && space_above >= calendar_height + margin;
    }
}

impl AppView {
    fn new() -> Self {
        Self {
            pickers: vec![
                DatePicker::new(),
                DatePicker::new(),
                DatePicker::new(),
                DatePicker::new(),
            ],
        }
    }

    fn render_picker(
        picker: &DatePicker,
        idx: usize,
        entity: Entity<Self>,
        offset_top: f32,
        note: &str,
    ) -> impl IntoElement {
        let date_str = picker.format_date();
        let is_open = picker.is_open;
        let show_above = picker.show_above;
        let month_days = picker.calendar.get_month_days();
        let weekdays = ["日", "一", "二", "三", "四", "五", "六"];
        let input_height = 40.0;
        // 与输入框的垂直间距，适当加大，避免遮挡输入框
        let popup_gap = 40.0;

        div()
            .mt(px(offset_top))
            .relative()
            .w(px(250.0))
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x6b7280))
                    .mb_1()
                    .child(note.to_string()),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .px_4()
                    .py_2()
                    .border(px(1.0))
                    .border_color(rgb(0xd1d5db))
                    .rounded_md()
                    .bg(rgb(0xffffff))
                    .cursor_pointer()
                    .hover(|style| style.border_color(rgb(0x3b82f6)))
                    .on_mouse_down(MouseButton::Left, {
                        let entity_toggle = entity.clone();
                        move |_, _window, cx| {
                            entity_toggle.update(cx, |app, cx| {
                                let picker = &mut app.pickers[idx];
                                // 简化：根据所在位置直接决定方向
                                // 上两行向下展开，下两行向上展开
                                picker.show_above = idx >= 2;
                                picker.is_open = !picker.is_open;
                                cx.notify();
                            });
                        }
                    })
                    .child(
                        div()
                            .flex_1()
                            .text_color(if picker.calendar.selected_date.is_some() {
                                rgb(0x111827)
                            } else {
                                rgb(0x9ca3af)
                            })
                            .child(date_str)
                    )
                    .child(
                        div()
                            .ml_2()
                            .text_color(rgb(0x6b7280))
                            .child("📅")
                    )
            )
            .child(
                if is_open {
                    let popup_base = if show_above {
                        div()
                            .absolute()
                            .bottom(px(input_height + popup_gap))
                            .left(px(0.0))
                            .mb_1()
                            .shadow_lg()
                    } else {
                        div()
                            .absolute()
                            .top(px(input_height + popup_gap))
                            .left(px(0.0))
                            .mt_1()
                            .shadow_lg()
                    };

                    popup_base.child(
                        div()
                            .flex()
                            .flex_col()
                            .w(px(350.0))
                            .border(px(1.0))
                            .border_color(rgb(0xd1d5db))
                            .rounded_lg()
                            .overflow_hidden()
                            .bg(rgb(0xffffff))
                            .child({
                                let entity_prev_year = entity.clone();
                                let entity_next_year = entity.clone();
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .px_4()
                                    .py_2()
                                    .h(px(40.0))
                                    .bg(rgb(0xf3f4f6))
                                    .border_b(px(1.0))
                                    .border_color(rgb(0xe5e7eb))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .w(px(32.0))
                                            .h(px(28.0))
                                            .rounded(px(4.0))
                                            .cursor_pointer()
                                            .hover(|style| style.bg(rgb(0xe5e7eb)))
                                            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                                entity_prev_year.update(cx, |app, cx| {
                                                    app.pickers[idx].calendar.prev_year();
                                                    cx.notify();
                                                });
                                            })
                                            .child("«")
                                    )
                                    .child(
                                        div()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(0x111827))
                                            .text_lg()
                                            .child(format!("{}年", picker.calendar.current_month.year()))
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .w(px(32.0))
                                            .h(px(28.0))
                                            .rounded(px(4.0))
                                            .cursor_pointer()
                                            .hover(|style| style.bg(rgb(0xe5e7eb)))
                                            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                                entity_next_year.update(cx, |app, cx| {
                                                    app.pickers[idx].calendar.next_year();
                                                    cx.notify();
                                                });
                                            })
                                            .child("»")
                                    )
                            })
                            .child({
                                let entity_prev = entity.clone();
                                let entity_next = entity.clone();
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .px_4()
                                    .py_2()
                                    .h(px(40.0))
                                    .bg(rgb(0xf9fafb))
                                    .border_b(px(1.0))
                                    .border_color(rgb(0xe5e7eb))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .w(px(32.0))
                                            .h(px(32.0))
                                            .rounded(px(4.0))
                                            .cursor_pointer()
                                            .hover(|style| style.bg(rgb(0xe5e7eb)))
                                            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                                entity_prev.update(cx, |app, cx| {
                                                    app.pickers[idx].calendar.prev_month();
                                                    cx.notify();
                                                });
                                            })
                                            .child("‹")
                                    )
                                    .child(
                                        div()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(rgb(0x111827))
                                            .child(format!("{}月", picker.calendar.current_month.month()))
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .w(px(32.0))
                                            .h(px(32.0))
                                            .rounded(px(4.0))
                                            .cursor_pointer()
                                            .hover(|style| style.bg(rgb(0xe5e7eb)))
                                            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                                entity_next.update(cx, |app, cx| {
                                                    app.pickers[idx].calendar.next_month();
                                                    cx.notify();
                                                });
                                            })
                                            .child("›")
                                    )
                            })
                            .child(
                                div()
                                    .flex()
                                    .h(px(40.0))
                                    .border_b(px(1.0))
                                    .border_color(rgb(0xe5e7eb))
                                    .children(weekdays.iter().map(|day| {
                                        div()
                                            .flex_1()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .py_2()
                                            .text_color(rgb(0x6b7280))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_sm()
                                            .child(*day)
                                    }))
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .children((0..6).map(|week| {
                                        div()
                                            .flex()
                                            .children((0..7).map(|day| {
                                                let idx_local = idx;
                                                let idx_cell = week * 7 + day;
                                                let date_opt = month_days.get(idx_cell).copied().flatten();
                                                
                                                if let Some(date) = date_opt {
                                                    let is_current = picker.calendar.is_current_month(&date);
                                                    let is_today = picker.calendar.is_today(&date);
                                                    let is_selected = picker.calendar.is_selected(&date);
                                                    let date_str = date.day().to_string();
                                                    let date_clone = date;
                                                    
                                                    div()
                                                        .flex_1()
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .h(px(40.0))
                                                        .relative()
                                                        .cursor_pointer()
                                                        .hover(move |style| {
                                                            if is_current {
                                                                style.bg(rgb(0xf3f4f6))
                                                            } else {
                                                                style
                                                            }
                                                        })
                                                        .on_mouse_down(MouseButton::Left, {
                                                            let entity_select = entity.clone();
                                                            let is_current_clone = is_current;
                                                            move |_, _, cx| {
                                                                if is_current_clone {
                                                                    entity_select.update(cx, |app, cx| {
                                                                        app.pickers[idx_local].calendar.selected_date = Some(date_clone);
                                                                        app.pickers[idx_local].is_open = false;
                                                                        cx.notify();
                                                                    });
                                                                }
                                                            }
                                                        })
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .justify_center()
                                                                .w(px(32.0))
                                                                .h(px(32.0))
                                                                .rounded(px(4.0))
                                                                .bg(if is_selected {
                                                                    rgb(0x3b82f6)
                                                                } else if is_today {
                                                                    rgb(0xeff6ff)
                                                                } else {
                                                                    rgb(0xffffff)
                                                                })
                                                                .text_color(if is_selected {
                                                                    rgb(0xffffff)
                                                                } else if !is_current {
                                                                    rgb(0xd1d5db)
                                                                } else if is_today {
                                                                    rgb(0x3b82f6)
                                                                } else {
                                                                    rgb(0x111827)
                                                                })
                                                                .font_weight(if is_today || is_selected {
                                                                    FontWeight::SEMIBOLD
                                                                } else {
                                                                    FontWeight::NORMAL
                                                                })
                                                                .child(date_str)
                                                        )
                                                        .into_any_element()
                                                } else {
                                                    div()
                                                        .flex_1()
                                                        .h(px(40.0))
                                                        .into_any_element()
                                                }
                                            }))
                                            .into_any_element()
                                    }))
                            )
                    )
                    .into_any_element()
                } else {
                    div().hidden().into_any_element()
                }
            )
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let entity = cx.entity();
        let offsets = [0.0, 0.0, 360.0, 360.0];
        let notes = [
            "示例1：应在下方展开",
            "示例2：应在下方展开",
            "示例3：靠近底部，可能上方展开",
            "示例4：靠近底部，可能上方展开",
        ];
        let any_open = self.pickers.iter().any(|p| p.is_open);
        let children = self
            .pickers
            .iter()
            .enumerate()
            .map(|(idx, picker)| AppView::render_picker(picker, idx, entity.clone(), offsets[idx], notes[idx]));

        div()
            .flex()
            .flex_wrap()
            .items_start()
            .justify_start()
            .gap_6()
            .p_6()
            .bg(rgb(0xf8fafc))
            .child(
                if any_open {
                    let entity_close = entity.clone();
                    div()
                        .absolute()
                        .top(px(0.0))
                        .left(px(0.0))
                        .right(px(0.0))
                        .bottom(px(0.0))
                        // 无背景色，仅用于捕获点击关闭弹层
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            entity_close.update(cx, |app, cx| {
                                for p in &mut app.pickers {
                                    p.is_open = false;
                                }
                                cx.notify();
                            });
                        })
                } else {
                    div().hidden()
                }
            )
            .children(children)
    }
}

fn main() {
    Application::new()
        .run(|cx: &mut App| {
            let bounds = Bounds::centered(None, size(px(900.0), px(700.0)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_window: &mut Window, cx: &mut App| {
                    App::new(cx, |_cx| AppView::new())
                },
            )
            .unwrap();
        });
}
