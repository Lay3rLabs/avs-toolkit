use avs_toolkit_shared::file::WasmFile;
use dominator_helpers::futures::AsyncLoader;
use web_sys::File;

use crate::{prelude::*, util::file::read_file_bytes};

pub struct WasmSourceUi {
    selected: Mutable<WasmChoice>,
    file_data: Mutable<Option<Arc<Vec<u8>>>>,
    url_data: Mutable<Option<String>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum WasmChoice {
    File,
    Url,
}

impl WasmSourceUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            selected: Mutable::new(WasmChoice::File),
            file_data: Mutable::new(None),
            url_data: Mutable::new(None),
        })
    }

    pub fn valid_signal(self: &Arc<Self>) -> impl Signal<Item = bool> {
        let state = self;

        self.selected
            .signal()
            .map(clone!(state => move |choice| {
                match choice {
                    WasmChoice::File => {
                        state.has_file_signal().boxed()
                    },
                    WasmChoice::Url => {
                        state.has_url_signal().boxed()
                    }
                }
            }))
            .flatten()
    }

    fn has_file_signal(self: &Arc<Self>) -> impl Signal<Item = bool> {
        self.file_data.signal_ref(|data| data.is_some())
    }
    fn has_url_signal(self: &Arc<Self>) -> impl Signal<Item = bool> {
        self.url_data.signal_ref(|data| data.is_some())
    }

    // returns the WasmFile and digest
    pub async fn extract(self: &Arc<Self>) -> Result<(WasmFile, Option<String>)> {
        let state = self;

        let file = match state.selected.get() {
            WasmChoice::File => match state.file_data.get_cloned() {
                None => {
                    bail!("No file selected");
                }
                Some(bytes) => WasmFile::Bytes(bytes.to_vec()),
            },
            WasmChoice::Url => match state.url_data.get_cloned() {
                None => {
                    bail!("No URL provided");
                }
                Some(url) => WasmFile::Url(url),
            },
        };

        let digest = match &file {
            WasmFile::Bytes(bytes) => None,
            WasmFile::Url(_) => {
                // in theory, calculate hash here
                None
            }
        };

        Ok((file, digest))
    }

    pub fn render(self: &Arc<Self>) -> Dom {
        let state = self;

        static CONTAINER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "row")
                .style("gap", "1rem")
                .style("align-items", "center")
            }
        });

        let dropdown = Dropdown::new()
            .with_intial_selected(Some(state.selected.get_cloned()))
            .with_options([
                ("File".to_string(), WasmChoice::File),
                ("URL".to_string(), WasmChoice::Url),
            ])
            .with_on_change(clone!(state => move |choice| {
                state.selected.set(*choice);
            }));

        html!("div", {
            .class(&*CONTAINER)
            .child(Label::new()
                .with_text("Wasm source")
                .with_direction(LabelDirection::Column)
                .render(dropdown.render())
            )
            .child_signal(state.selected.signal_cloned().map(clone!(state => move |choice| {
                Some(match choice {
                    WasmChoice::File => {
                        state.render_file()
                    },
                    WasmChoice::Url => {
                        state.render_url()
                    }
                })
            })))
        })
    }

    fn render_file(self: &Arc<Self>) -> Dom {
        let state = self;

        let file_loader = AsyncLoader::new();

        static CONTAINER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("gap", "1rem")
            }
        });

        html!("div", {
            .class(&*CONTAINER)
            .child(Label::new()
                .with_text("Choose a .wasm file")
                .with_direction(LabelDirection::Column)
                .render(html!("input" => web_sys::HtmlInputElement, {
                    .attrs!{
                        "type": "file",
                        "id": "wasm-upload",
                        "accept": ".wasm"
                    }
                    .with_node!(elem => {
                        .event(clone!(elem, state, file_loader => move |evt:events::Change| {
                            if let Some(file) = elem.files().and_then(|files| files.item(0)) {
                                file_loader.load(clone!(state => async move {
                                    match read_file_bytes(&file).await {
                                        Ok(bytes) => {
                                            state.file_data.set(Some(Arc::new(bytes)));
                                        },
                                        Err(err) => {
                                            log::error!("Error reading file: {:?}", err);
                                        }
                                    }
                                }));
                            }
                        }))
                    })
                }))
            )
            .child_signal(file_loader.is_loading().map(|is_loading| {
                match is_loading {
                    true => Some(html!("div", {
                        .class(FontSize::Body.class())
                        .text("Loading...")
                    })),
                    false => None
                }
            }))
        })
    }

    fn render_url(self: &Arc<Self>) -> Dom {
        let state = self;

        Label::new()
            .with_text("URL to .wasm")
            .with_direction(LabelDirection::Column)
            .render(
                TextInput::new()
                    .with_placeholder("e.g. http://example.com/foo.wasm")
                    .with_mixin(|dom| dom.style("width", "30rem"))
                    .with_on_input(clone!(state => move |url| {
                        state.url_data.set_neq(url);
                    }))
                    .render(),
            )
    }
}
