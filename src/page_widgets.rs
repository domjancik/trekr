use crate::actions::ActionSource;
use crate::app::{App, AppControl, DiscoverabilityTarget};
use crate::pages::AppPage;
use sdl3::rect::Rect;
use sdl3::render::{Canvas, RenderTarget};

pub(crate) trait PageWidget {
    fn render<T: RenderTarget>(
        &self,
        app: &App,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn handle_pointer(
        &self,
        app: &mut App,
        content_bounds: Rect,
        x: i32,
        y: i32,
        source: ActionSource,
    ) -> Option<AppControl>;
    fn discoverability_targets(
        &self,
        app: &App,
        content_bounds: Rect,
    ) -> Vec<(Rect, DiscoverabilityTarget)>;
}

struct TimelinePageWidget;
struct MappingsPageWidget;
struct MidiIoPageWidget;
struct RoutingPageWidget;

impl PageWidget for TimelinePageWidget {
    fn render<T: RenderTarget>(
        &self,
        app: &App,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        app.draw_timeline_page(canvas, content_bounds)
    }

    fn handle_pointer(
        &self,
        app: &mut App,
        content_bounds: Rect,
        x: i32,
        y: i32,
        source: ActionSource,
    ) -> Option<AppControl> {
        app.handle_timeline_pointer(content_bounds, x, y, source)
    }

    fn discoverability_targets(
        &self,
        app: &App,
        content_bounds: Rect,
    ) -> Vec<(Rect, DiscoverabilityTarget)> {
        app.timeline_discoverability_targets(content_bounds)
    }
}

impl PageWidget for MappingsPageWidget {
    fn render<T: RenderTarget>(
        &self,
        app: &App,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        app.draw_mappings_page(canvas, content_bounds)
    }

    fn handle_pointer(
        &self,
        app: &mut App,
        content_bounds: Rect,
        x: i32,
        y: i32,
        source: ActionSource,
    ) -> Option<AppControl> {
        app.handle_mappings_pointer(content_bounds, x, y, source)
    }

    fn discoverability_targets(
        &self,
        _app: &App,
        _content_bounds: Rect,
    ) -> Vec<(Rect, DiscoverabilityTarget)> {
        Vec::new()
    }
}

impl PageWidget for MidiIoPageWidget {
    fn render<T: RenderTarget>(
        &self,
        app: &App,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        app.draw_midi_io_page(canvas, content_bounds)
    }

    fn handle_pointer(
        &self,
        app: &mut App,
        content_bounds: Rect,
        x: i32,
        y: i32,
        source: ActionSource,
    ) -> Option<AppControl> {
        app.handle_midi_io_pointer(content_bounds, x, y, source)
    }

    fn discoverability_targets(
        &self,
        _app: &App,
        _content_bounds: Rect,
    ) -> Vec<(Rect, DiscoverabilityTarget)> {
        Vec::new()
    }
}

impl PageWidget for RoutingPageWidget {
    fn render<T: RenderTarget>(
        &self,
        app: &App,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        app.draw_routing_page(canvas, content_bounds)
    }

    fn handle_pointer(
        &self,
        app: &mut App,
        content_bounds: Rect,
        x: i32,
        y: i32,
        source: ActionSource,
    ) -> Option<AppControl> {
        app.handle_routing_pointer(content_bounds, x, y, source)
    }

    fn discoverability_targets(
        &self,
        app: &App,
        content_bounds: Rect,
    ) -> Vec<(Rect, DiscoverabilityTarget)> {
        app.routing_discoverability_targets(content_bounds)
    }
}

pub(crate) fn render_page<T: RenderTarget>(
    page: AppPage,
    app: &App,
    canvas: &mut Canvas<T>,
    content_bounds: Rect,
) -> Result<(), Box<dyn std::error::Error>> {
    match page {
        AppPage::Timeline => TimelinePageWidget.render(app, canvas, content_bounds),
        AppPage::Mappings => MappingsPageWidget.render(app, canvas, content_bounds),
        AppPage::MidiIo => MidiIoPageWidget.render(app, canvas, content_bounds),
        AppPage::Routing => RoutingPageWidget.render(app, canvas, content_bounds),
    }
}

pub(crate) fn handle_page_pointer(
    page: AppPage,
    app: &mut App,
    content_bounds: Rect,
    x: i32,
    y: i32,
    source: ActionSource,
) -> Option<AppControl> {
    match page {
        AppPage::Timeline => TimelinePageWidget.handle_pointer(app, content_bounds, x, y, source),
        AppPage::Mappings => MappingsPageWidget.handle_pointer(app, content_bounds, x, y, source),
        AppPage::MidiIo => MidiIoPageWidget.handle_pointer(app, content_bounds, x, y, source),
        AppPage::Routing => RoutingPageWidget.handle_pointer(app, content_bounds, x, y, source),
    }
}

pub(crate) fn page_discoverability_targets(
    page: AppPage,
    app: &App,
    content_bounds: Rect,
) -> Vec<(Rect, DiscoverabilityTarget)> {
    match page {
        AppPage::Timeline => TimelinePageWidget.discoverability_targets(app, content_bounds),
        AppPage::Mappings => MappingsPageWidget.discoverability_targets(app, content_bounds),
        AppPage::MidiIo => MidiIoPageWidget.discoverability_targets(app, content_bounds),
        AppPage::Routing => RoutingPageWidget.discoverability_targets(app, content_bounds),
    }
}
