#include "framebuffer.h"
#include "dashboard.h"

// Main kernel entry point
void kernel_main(void) {
    framebuffer_t fb;
    dashboard_t dash;
    
    // Initialize framebuffer (1920x1080)
    if (fb_init(&fb, 1920, 1080) != 0) {
        // Fallback to 1280x720
        fb_init(&fb, 1280, 720);
    }
    
    // Initialize dashboard
    dashboard_init(&dash, "LibreDash Demo");
    
    // Create sample dashboard elements
    dash_element_t gauge1 = {
        .type = DASH_ELEMENT_GAUGE,
        .x = 50, .y = 50,
        .width = 400, .height = 60,
        .color = COLOR_GREEN,
        .value = 75.0f,
        .min_value = 0.0f,
        .max_value = 100.0f
    };
    dashboard_add_element(&dash, &gauge1);
    
    dash_element_t gauge2 = {
        .type = DASH_ELEMENT_GAUGE,
        .x = 50, .y = 130,
        .width = 400, .height = 60,
        .color = COLOR_BLUE,
        .value = 45.0f,
        .min_value = 0.0f,
        .max_value = 100.0f
    };
    dashboard_add_element(&dash, &gauge2);
    
    dash_element_t value1 = {
        .type = DASH_ELEMENT_VALUE,
        .x = 500, .y = 50,
        .width = 200, .height = 60,
        .color = COLOR_CYAN,
        .value = 65.0f,
        .min_value = 0.0f,
        .max_value = 100.0f
    };
    dashboard_add_element(&dash, &value1);
    
    dash_element_t graph1 = {
        .type = DASH_ELEMENT_GRAPH,
        .x = 50, .y = 250,
        .width = 650, .height = 200,
        .color = COLOR_YELLOW,
        .value = 0.0f,
        .min_value = 0.0f,
        .max_value = 100.0f
    };
    dashboard_add_element(&dash, &graph1);
    
    dash_element_t label1 = {
        .type = DASH_ELEMENT_LABEL,
        .x = 50, .y = 500,
        .width = 650, .height = 50,
        .color = COLOR_WHITE
    };
    dashboard_add_element(&dash, &label1);
    
    // Render dashboard
    dashboard_render(&dash, &fb);
    
    // Animation loop - update gauges
    float counter = 0.0f;
    while (1) {
        counter += 0.5f;
        if (counter > 100.0f) counter = 0.0f;
        
        // Update gauge values
        dashboard_update_value(&dash, 0, counter);
        dashboard_update_value(&dash, 1, 100.0f - counter);
        dashboard_update_value(&dash, 2, (counter * 0.7f));
        
        // Re-render
        dashboard_render(&dash, &fb);
        
        // Simple delay
        for (volatile int i = 0; i < 1000000; i++);
    }
}
