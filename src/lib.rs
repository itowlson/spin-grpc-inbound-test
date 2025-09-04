use routeguide::route_guide_server::RouteGuide;
use routeguide::route_guide_server::RouteGuideServer;
pub mod routeguide {
    tonic::include_proto!("routeguide");
}

struct MyHttpServer;

wasi::http::proxy::export!(MyHttpServer);

impl wasi::exports::http::incoming_handler::Guest for MyHttpServer {
    fn handle(request: wasi::exports::http::incoming_handler::IncomingRequest, response_out: wasi::exports::http::incoming_handler::ResponseOutparam) {
        let registry = wasi_hyperium::poll::Poller::default();
        let server = RouteGuideServer::new(Svc);
        let e = wasi_hyperium::hyperium1::handle_service_call(server, request, response_out, registry);
        e.unwrap();
    }
}

struct Svc;

#[tonic::async_trait]
impl RouteGuide for Svc {
    async fn get_feature(&self, request: tonic::Request<routeguide::Point>) -> std::result::Result<tonic::Response<routeguide::Feature>, tonic::Status> {
        let lat = request.get_ref().latitude;
        let mut feat = routeguide::Feature::default();
        if lat < 0 {
            feat.name = "west".to_owned();
        } else if lat > 0 {
            feat.name = "east".to_owned();
        } else {
            feat.name = "on the meridian".to_owned();
        }
        Ok(tonic::Response::new(feat))
    }

    type ListFeaturesStream = std::pin::Pin<Box<dyn futures::Stream<Item=Result<routeguide::Feature,tonic::Status> > + Send + 'static >>;

    async fn list_features(&self, _request: tonic::Request<routeguide::Rectangle>) -> std::result::Result<tonic::Response<Self::ListFeaturesStream> ,tonic::Status> {
        let stm = async_stream::stream! {
            let pages = [
                "https://www.fermyon.com/blog/sqlite-is-edge-scale",
                "https://www.fermyon.com/blog/openapi-docs-for-spin-with-rust",
                "https://www.fermyon.com/blog/building-a-graphql-api-with-fwf"
            ];

            for page in pages {
                let req = spin_sdk::http::Request::get(page).build();
                let fut = spin_sdk::http::send::<spin_sdk::http::Request, spin_sdk::http::Response>(req);
                let resp = spin_executor::run(fut);

                yield match resp {
                    Ok(r) => Ok(routeguide::Feature { name: format!("{page} has content-len {:?}", r.header("content-length").and_then(|hv| hv.as_str())), location: None }),
                    Err(e) => Err(tonic::Status::from_error(Box::new(e)))
                }
            }
        };

        Ok(tonic::Response::new(Box::pin(stm)))
    }

    async fn record_route(&self, request: tonic::Request<tonic::Streaming<routeguide::Point>>) -> std::result::Result<tonic::Response<routeguide::RouteSummary>, tonic::Status> {
        let mut req = request;
        let r = req.get_mut();

        let mut distance = 0;
        let mut count = 0;
        let mut last_pt = None;

        loop {
            let Some(pt) = r.message().await? else {
                break;
            };

            count = count + 1;

            if let Some(last) = last_pt {
                distance = distance + dist(last, pt);
            }

            last_pt = Some(pt);
        }

        Ok(tonic::Response::new(routeguide::RouteSummary { point_count: count, feature_count: 0, distance, elapsed_time: 0 }))
    }

    type RouteChatStream = std::pin::Pin<Box<dyn futures::Stream<Item=Result<routeguide::RouteNote,tonic::Status> > + Send + 'static >>;

    async fn route_chat(&self, request: tonic::Request<tonic::Streaming<routeguide::RouteNote>>) -> std::result::Result<tonic::Response<Self::RouteChatStream>, tonic::Status> {
        use futures::StreamExt;

        let stm = request.into_inner().flat_map(|m| {
            async_stream::stream! {
                match m {
                    Err(e) => { yield Err(e); }
                    Ok(m) => { for i in 0..3 {
                        yield Ok(routeguide::RouteNote { message: format!("Note {i} for message {}", m.message), location: None });
                        std::thread::sleep(std::time::Duration::from_millis(500));
                    }}
                }
            }
        });

        Ok(tonic::Response::new(Box::pin(stm)))
    }
}

fn dist(pt1: routeguide::Point, pt2: routeguide::Point) -> i32 {
    let latd = pt1.latitude - pt2.latitude;
    let longd = pt1.longitude - pt2.longitude;
    let d = f64::sqrt((latd * latd + longd * longd) as f64);
    d as i32
}
