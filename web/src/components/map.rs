use dioxus::prelude::*;
use game::areas::AreaDetails;
use std::collections::HashMap;

#[component]
pub fn Map(areas: Vec<AreaDetails>) -> Element {
    let map_areas: HashMap<String, bool> = areas.clone().into_iter().map(|area| {
        (area.clone().area.clone().unwrap().to_string().to_lowercase(), area.is_open())
    }).collect();

    rsx! {
        svg {
            view_box: "0 0 806 806",
            path {
                id: "cornucopia",
                "data-open": map_areas["cornucopia"],
                class: "fill-stone-200 data-[open=false]:fill-red-500 theme3:fill-stone-400",
                d: "m 402.95508,220.7032 c 100.448,0 182,81.551 182,182 0,100.448 -81.552,182 -182,182 -100.449,0 -182,-81.552 -182,-182 0,-100.449 81.551,-182 182,-182 z" }
            path {
                id: "north",
                "data-open": map_areas["north"],
                class: "fill-stone-200 data-[open=false]:fill-red-500 theme3:fill-stone-400",
                d: "M 679.75202,109.23829 532.26179,256.90236 c -34.26561,-30.11954 -79.13428,-48.44727 -128.21289,-48.44727 -49.16061,0 -94.09928,18.38777 -128.38672,48.59766 L 128.00007,109.56642 C 202.62746,39.348491 301.2721,1.1157135e-6 404.0489,1.1157135e-6 506.66643,1.1157135e-6 605.16579,39.225021 679.75202,109.23829 Z" }
            path {
                id: "south",
                "data-open": map_areas["south"],
                class: "fill-stone-200 data-[open=false]:fill-red-500 theme3:fill-stone-400",
                d: "M 128.00007,695.81446 275.4903,548.15039 c 34.26561,30.11954 79.13428,48.44727 128.21289,48.44727 49.16061,0 94.09928,-18.38777 128.38672,-48.59766 l 147.66211,147.48633 c -74.62739,70.21793 -173.27203,109.56642 -276.04883,109.56642 -102.61753,0 -201.11689,-39.22502 -275.70312,-109.23829 z" }
            path {
                id: "west",
                "data-open": map_areas["west"],
                class: "fill-stone-200 data-[open=false]:fill-red-500 theme3:fill-stone-400",
                d: "m 109.23828,127.00008 147.66407,147.49023 c -30.11954,34.26561 -48.44727,79.13428 -48.44727,128.21289 0,49.16061 18.38777,94.09928 48.59766,128.38672 L 109.56641,678.75203 C 39.348483,604.12464 1.9239319e-6,505.48 -7.6068148e-8,402.7032 1.9239319e-6,300.08567 39.225014,201.58631 109.23828,127.00008 Z" }
            path {
                id: "east",
                "data-open": map_areas["east"],
                class: "fill-stone-200 data-[open=false]:fill-red-500 theme3:fill-stone-400",
                d: "M 696.81446,678.75203 549.15039,531.2618 c 30.11954,-34.26561 48.44727,-79.13428 48.44727,-128.21289 0,-49.16061 -18.38777,-94.09928 -48.59766,-128.38672 L 696.48633,127.00008 c 70.21793,74.62739 109.56641,173.27203 109.56641,276.04883 0,102.61753 -39.22501,201.11689 -109.23828,275.70312 z" }
        }
    }
}
