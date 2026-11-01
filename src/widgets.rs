use egui::*;
use egui_alignments::Alignable;

pub fn circular_button(ui: &mut Ui, diameter: f32, text: impl Into<WidgetText>) -> Response {
    let radius = diameter / 2.0;
    let size = Vec2::splat(diameter);
    // 1. Upgrade Sense to click_and_drag so is_pointer_button_down_on works reliably
    let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());

    let center = rect.center();
    let painter = ui.painter();
    let widgets = &ui.style().visuals.widgets;

    // 2. Select the visual style matching the current state
    let visuals = if response.is_pointer_button_down_on() {
        &widgets.active
    } else if response.hovered() {
        &widgets.hovered
    } else {
        &widgets.inactive
    };

    // 3. Draw the background circle using the theme's weak_bg_fill
    painter.circle_filled(center, radius, visuals.weak_bg_fill);

    // 4. Draw the native stroke border matching egui's styling rules
    painter.circle_stroke(center, radius, visuals.bg_stroke);

    // 5. Center and render the text atom using the theme's text color
    let widget_text: WidgetText = text.into();
    let galley = widget_text.into_galley(ui, None, diameter, TextStyle::Button);

    // Position text directly in the center of the rectangle
    let text_pos = center - galley.size() / 2.0;
    painter.galley(text_pos, galley, visuals.text_color());

    response
}

pub fn sidebar_entry<T: PartialEq>(ui: &mut Ui, current_value: &mut T, set_value: T, text: impl Into<String>) {
    let is_selected = *current_value == set_value;
    if sized_entry_centered(ui, 40.0, is_selected, text).clicked() {
        *current_value = set_value;
    }
}

pub fn sized_entry_centered(ui: &mut Ui, h: f32, is_selected: bool, text: impl Into<String>) -> Response {
    sized_entry(ui, h, Align2::CENTER_TOP, is_selected, text)
}

pub fn sized_entry(ui: &mut Ui, h: f32, align: Align2, is_selected: bool, text: impl Into<String>) -> Response {
    let w = ui.available_width();

    Button::selectable(is_selected, RichText::new(text).size(20.0)).min_size(Vec2::new(w, h)).align(ui, align)
}
