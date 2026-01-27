use crate::framebuffer::{Framebuffer, COLOR_BLACK, COLOR_WHITE, COLOR_RED, COLOR_GREEN, COLOR_YELLOW, COLOR_GRAY};

const MAX_DASHBOARD_ELEMENTS: usize = 32;

#[derive(Copy, Clone, PartialEq)]
pub enum DashElementType {
    Gauge,
    Label,
    Graph,
    Value,
}

#[derive(Copy, Clone)]
pub struct DashElement {
    pub element_type: DashElementType,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub color: u32,
    pub label: [u8; 64],
    pub value: f32,
    pub min_value: f32,
    pub max_value: f32,
}

pub struct Dashboard {
    name: [u8; 128],
    elements: [Option<DashElement>; MAX_DASHBOARD_ELEMENTS],
    element_count: usize,
}

impl Dashboard {
    pub fn new(name: &str) -> Self {
        let mut name_bytes = [0u8; 128];
        let bytes = name.as_bytes();
        let len = bytes.len().min(127);
        name_bytes[..len].copy_from_slice(&bytes[..len]);
        
        Dashboard {
            name: name_bytes,
            elements: [None; MAX_DASHBOARD_ELEMENTS],
            element_count: 0,
        }
    }
    
    pub fn add_element(&mut self, element: DashElement) {
        if self.element_count < MAX_DASHBOARD_ELEMENTS {
            self.elements[self.element_count] = Some(element);
            self.element_count += 1;
        }
    }
    
    pub fn update_value(&mut self, element_id: usize, value: f32) {
        if element_id < self.element_count {
            if let Some(ref mut elem) = self.elements[element_id] {
                elem.value = value;
            }
        }
    }
    
    pub fn render(&self, fb: &mut Framebuffer) {
        // Clear background
        fb.clear(COLOR_BLACK);
        
        // Render each element
        for i in 0..self.element_count {
            if let Some(elem) = self.elements[i] {
                match elem.element_type {
                    DashElementType::Gauge => self.render_gauge(&elem, fb),
                    DashElementType::Label => self.render_label(&elem, fb),
                    DashElementType::Value => self.render_value(&elem, fb),
                    DashElementType::Graph => self.render_graph(&elem, fb),
                }
            }
        }
    }
    
    fn render_gauge(&self, elem: &DashElement, fb: &mut Framebuffer) {
        // Draw gauge background
        fb.draw_filled_rect(elem.x, elem.y, elem.width, elem.height, COLOR_BLACK);
        fb.draw_rect(elem.x, elem.y, elem.width, elem.height, elem.color);
        
        // Calculate fill based on value
        let mut percentage = (elem.value - elem.min_value) / (elem.max_value - elem.min_value);
        if percentage < 0.0 {
            percentage = 0.0;
        }
        if percentage > 1.0 {
            percentage = 1.0;
        }
        
        let fill_width = (percentage * (elem.width - 4) as f32) as u32;
        if fill_width > 0 && elem.height > 4 {
            fb.draw_filled_rect(elem.x + 2, elem.y + 2, fill_width, elem.height - 4, elem.color);
        }
    }
    
    fn render_label(&self, elem: &DashElement, fb: &mut Framebuffer) {
        // Draw label box
        fb.draw_filled_rect(elem.x, elem.y, elem.width, elem.height, COLOR_BLACK);
        fb.draw_rect(elem.x, elem.y, elem.width, elem.height, elem.color);
    }
    
    fn render_value(&self, elem: &DashElement, fb: &mut Framebuffer) {
        // Draw value display
        fb.draw_filled_rect(elem.x, elem.y, elem.width, elem.height, COLOR_BLACK);
        fb.draw_rect(elem.x, elem.y, elem.width, elem.height, elem.color);
        
        // Draw colored indicator based on value range
        let percentage = (elem.value - elem.min_value) / (elem.max_value - elem.min_value);
        let indicator_color = if percentage > 0.8 {
            COLOR_RED
        } else if percentage > 0.6 {
            COLOR_YELLOW
        } else {
            COLOR_GREEN
        };
        
        if elem.width > 10 && elem.height > 10 {
            fb.draw_filled_rect(elem.x + 5, elem.y + 5, 20, elem.height - 10, indicator_color);
        }
    }
    
    fn render_graph(&self, elem: &DashElement, fb: &mut Framebuffer) {
        // Draw graph background
        fb.draw_filled_rect(elem.x, elem.y, elem.width, elem.height, COLOR_BLACK);
        fb.draw_rect(elem.x, elem.y, elem.width, elem.height, elem.color);
        
        // Draw grid lines
        for i in 1..4 {
            let y_pos = elem.y + (elem.height * i) / 4;
            let mut x = elem.x + 2;
            while x < elem.x + elem.width - 2 {
                fb.draw_pixel(x, y_pos, COLOR_GRAY);
                x += 4;
            }
        }
    }
}

// TODO: Implement .dash format parser
// This would parse JSON-based .dash files for dashboard configuration
pub fn load_dashboard_from_dash(_dash_data: &str) -> Option<Dashboard> {
    None
}
