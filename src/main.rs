use leptos::*;

mod data;
use data::Data;

fn main() -> anyhow::Result<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Trace)
        .map_err(|_| anyhow::anyhow!("failed to initialize logger."))?;
    leptos::mount_to_body(|| view! { <App/> });
    Ok(())
}

#[component]
fn App() -> impl IntoView {
    let (latitude, set_latitude) = create_signal(48.3);
    let (longitude, set_longitude) = create_signal(16.3);
    let (forecast_days, set_forecast_days) = create_signal(2);

    let data = create_resource(
        move || (latitude.get(), longitude.get(), forecast_days.get()),
        move |(latitude, longitude, forecast_days)| async move {
            Data::load(latitude, longitude, forecast_days).await
        },
    );

    view! {
        // Present the data
        {move || match data.get() {
            None => view! { <p>"Loading..."</p> }.into_view(),
            Some(data) => view! {
                <ErrorBoundary
                    // the fallback receives a signal containing current errors
                    fallback=|errors| view! {
                        <div class="error">
                            <p>"Your query has problems! Errors: "</p>
                            // we can render a list of errors as strings, if we'd like
                            <ul>
                                {move || errors.get()
                                    .into_iter()
                                    .map(|(_, e)| view! { <li>{e.to_string()}</li>})
                                    .collect_view()
                                }
                            </ul>
                        </div>
                    }
                >
                    <div>{data}</div    >
                </ErrorBoundary>
            }.into_view(),
        }}
        <form on:submit=move |_| {data.refetch();}>
            <fieldset>
                <legend>"Forecast days: " {move || forecast_days.get()}</legend>
                <input type="number"
                    min="1"
                    max="16"
                    on:input = move |ev| {
                        let v = event_target_value(&ev).parse().unwrap_or(2);
                        log::trace!("setting forcast days to {v}");
                        // event_target_value is a Leptos helper function
                        // it functions the same way as event.target.value
                        // in JavaScript, but smooths out some of the typecasting
                        // necessary to make this work in Rust
                        set_forecast_days.set(v);
                    }
                    // the `prop:` syntax lets you update a DOM property,
                    // rather than an attribute.
                    prop:value=forecast_days
                />
            </fieldset>
            <fieldset>
                <legend>"Location! 🗺: " {move || latitude.get()} ", " {move || longitude.get()}</legend>
                // <label for="city">"City: "</label>
                // <select
                //     id="city"
                //     on:input=move |ev| {
                //         let (lat, long) = event_target_value(&ev).parse::<(f64, f64)>().unwrap();
                //         set_latitude.set(lat);
                //         set_longitude.set(long);
                //     }
                //     prop:value=(latitude, longitude)
                // >
                //     <option value="(48.3, 16.3)">Vienna</option>
                //     <option value="">"Other"</option>
                // </select>
                <label for="latitude">"latitude: "</label>
                <input type="range"
                    id="latitude"
                    min="-90"
                    max="90"
                    step="0.1"
                    on:input = move |ev| {
                        let v = event_target_value(&ev).parse().unwrap_or(48.3);
                        log::trace!("setting latitude days to {v}");
                        set_latitude.set(v);
                    }
                    prop:value=latitude
                />
                <label for="longitude">"longitude: "</label>
                <input type="range"
                    id="longitude"
                    min="0"
                    max="180"
                    step="0.1"
                    on:input = move |ev| {
                        let v = event_target_value(&ev).parse().unwrap_or(16.3);
                        log::trace!("setting longitude days to {v}");
                        set_longitude.set(v);
                    }
                    prop:value=longitude
                />
            </fieldset>
        </form>

    }
}
