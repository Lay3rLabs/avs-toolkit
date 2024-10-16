use crate::prelude::*;

pub struct LogoSvg {}

impl LogoSvg {
    pub fn render() -> Dom {
        static SVG: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("width", "1.5rem")
                .style("height", "1.5rem")
            }
        });
        svg!("svg", {
            .class(&*SVG)
            .attrs!{
                "viewBox": "0 0 24 24",
                "fill": "none",
                "xmlns": "http://www.w3.org/2000/svg",
            }
            .children([
                svg!("defs", {
                    .child(
                        svg!("radialGradient", {
                            .attrs!{
                                "id": "paint0_radial_2072_730",
                                "cx": "0",
                                "cy": "0",
                                "r": "1",
                                "gradientUnits": "userSpaceOnUse",
                                "gradientTransform": "translate(1.0184 5.60542) rotate(30.4154) scale(40.7782 65.2419)",
                            }
                            .children([
                                svg!("stop", {
                                    .attr("stop-color", "#D72DE5")
                                }),
                                svg!("stop", {
                                    .attrs!{
                                        "offset": "0.1093",
                                        "stop-color": "#D62EE5",
                                    }
                                }),
                                svg!("stop", {
                                    .attrs!{
                                        "offset": "0.198245",
                                        "stop-color": "#D232E5",
                                    }
                                }),
                                svg!("stop", {
                                    .attrs!{
                                        "offset": "0.269967",
                                        "stop-color": "#CC38E4",
                                    }
                                }),
                                svg!("stop", {
                                    .attrs!{
                                        "offset": "0.327597",
                                        "stop-color": "#C540E3",
                                    }
                                }),
                                svg!("stop", {
                                    .attrs!{
                                        "offset": "0.374267",
                                        "stop-color": "#BC49E2",
                                    }
                                }),
                                svg!("stop", {
                                    .attrs!{
                                        "offset": "0.413108",
                                        "stop-color": "#B154E1",
                                    }
                                }),
                                svg!("stop", {
                                    .attrs!{
                                        "offset": "0.447252",
                                        "stop-color": "#A561E0",
                                    }
                                }),
                                svg!("stop", {
                                    .attrs!{
                                        "offset": "0.479831",
                                        "stop-color": "#996DDF",
                                    }
                                }),
                                svg!("stop", {
                                    .attrs!{
                                        "offset": "0.513975",
                                        "stop-color": "#8C7BDE",
                                    }
                                }),
                                svg!("stop", {
                                    .attrs!{
                                        "offset": "0.552816",
                                        "stop-color": "#7F89DD",
                                    }
                                }),
                                svg!("stop", {
                                    .attrs!{
                                        "offset": "0.599486",
                                        "stop-color": "#7197DC",
                                    }
                                }),
                                svg!("stop", {
                                    .attrs!{
                                        "offset": "0.657117",
                                        "stop-color": "#64A5DA",
                                    }
                                }),
                                svg!("stop", {
                                    .attrs!{
                                        "offset": "0.728839",
                                        "stop-color": "#57B2D9",
                                    }
                                }),
                                svg!("stop", {
                                    .attrs!{
                                        "offset": "0.817784",
                                        "stop-color": "#4BBED8",
                                    }
                                }),
                                svg!("stop", {
                                    .attrs!{
                                        "offset": "0.927083",
                                        "stop-color": "#40CAD7",
                                    }
                                }),
                            ])
                        })
                    )
                }),
                svg!("path", {
                    .attrs!{
                        "d": "M0 12C0 5.37258 5.37258 0 12 0V0C18.6274 0 24 5.37258 24 12V12C24 18.6274 18.6274 24 12 24V24C5.37258 24 0 18.6274 0 12V12Z",
                        "fill": "url(#paint0_radial_2072_730)",
                    }
                }),
            ])
        })
    }
}
