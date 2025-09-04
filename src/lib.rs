// use spin_sdk::http::{IncomingRequest, IntoResponse, Request, Response, ResponseOutparam};
// use spin_sdk::http_component;

pub mod executor_fork;

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

// /// A simple Spin HTTP component.
// #[http_component]
// fn handle_grpc_inbound_test(request: IncomingRequest, response_out: ResponseOutparam) {
//     let registry = wasi_hyperium::poll::Poller::default();
//     let mut server = RouteGuideServer::new(Svc);
//     let e = wasi_hyperium::hyperium1::handle_service_call(server, request, response_out, registry);
//     e.unwrap();
// }

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

    // type ListFeaturesStream = ThingStream2<TheNextInator>; // ThingStream<routeguide::Feature>; // tokio_stream::wrappers::ReceiverStream<Result<routeguide::Feature, tonic::Status>>;
    type ListFeaturesStream = ThingStream3<routeguide::Feature>;

    async fn list_features(&self, _request: tonic::Request<routeguide::Rectangle>) -> std::result::Result<tonic::Response<Self::ListFeaturesStream> ,tonic::Status> {
        // let (tx, rx) = tokio::sync::mpsc::channel(4);
        // let stm = tokio_stream::wrappers::ReceiverStream::new(rx);

        // let closure = || {
        //     if wasi::random::random::get_random_u64() < u64::MAX / 4 {
        //         None
        //     } else {
        //         Some(routeguide::Feature { name: "arsebiscuit".to_owned(), location: None })
        //     }
        // };

        // let stm = ThingStream {
        //     it: Box::new(closure),
        // };

        let mut nexter = AnotherNextinator::new();
        let closure = move || { nexter.next() };
        let stm = ThingStream3 {
            it: Box::new(closure),
        };

        // ugh this is not so good.  tonic wants us to return the stm but have a spawned task
        // populating it.  I am not sure of the practicalities of that in Spin.  We can totally
        // do the streaming response thing by hand but I think we may need a custom stm type
        // that you construct with like a closure or something

        Ok(tonic::Response::new(stm))
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

    type RouteChatStream = std::pin::Pin<Box<dyn futures::Stream<Item=Result<routeguide::RouteNote,tonic::Status> > + Send + 'static >>; // ThingStream4<routeguide::RouteNote>;

    async fn route_chat(&self, request: tonic::Request<tonic::Streaming<routeguide::RouteNote>>) -> std::result::Result<tonic::Response<Self::RouteChatStream>, tonic::Status> {

        use futures::StreamExt;

        let innn = request.into_inner();
        let stmmmmmmm = innn.flat_map(|m| {
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

        let whositt = Box::pin(stmmmmmmm);

        return Ok(tonic::Response::new(whositt));

        // --------------------------------------------------------------------------------------

        // let stmmm = async_stream::stream! {
        //     for i in 0..3 {
        //         if i > 10 {
        //             yield Err(tonic::Status::new(tonic::Code::Unknown, "fie"));
        //         }
        //         yield Ok(routeguide::RouteNote { message: format!("arse {i}"), location: None });
        //         std::thread::sleep(std::time::Duration::from_millis(500));
        //     }
        // };

        // let whosit = Box::pin(stmmm);

        // return Ok(tonic::Response::new(whosit))

        // --------------------------------------------------------------------------------------

        // let (tx, rx) = std::sync::mpsc::channel::<routeguide::RouteNote>();

        // let fut = async move {
        //     let mut n = 0;
        //     loop {
        //         n = n + 1;
        //         if n > 3 {
        //             eprintln!("cheating, exit loop");
        //             break;
        //         }
        //         eprintln!("looking for msg");
        //         let Ok(Some(r)) = request.get_mut().message().await else {
        //             eprintln!("stahp");
        //             break;
        //         };
        //         eprintln!("got one");
        //         if tx.send(r).is_err() {
        //             break;
        //         }
        //     }
        // };
        // let fut = Box::pin(fut);

        // let closure = move || {
        //     eprintln!("checking rx");
        //     let Ok(m) = rx.recv() else {
        //         eprintln!("nope, stoppy time");
        //         return None;
        //     };
        //     eprintln!("rxed something, answer and continue");
        //     Some(Ok(routeguide::RouteNote { location: m.location, message: format!("Hello {}", m.message) }))
        // };

        // let stm = ThingStream4 {
        //     fut,
        //     it: Box::new(closure),
        // };

        // Ok(tonic::Response::new(stm))
    }
}

#[pin_project::pin_project]
struct ThingStream4<T> {
    #[pin]
    fut: std::pin::Pin<Box<dyn core::future::Future<Output = ()> + Send + 'static>>,
    it: Box<dyn (FnMut() -> Option<std::result::Result<T, tonic::Status>>) + Send>,
}
// impl<T> ThingStream4<T> {
//     fn thingy(&mut self) -> &mut dyn core::future::Future<Output = ()> {
//         self.fut.as_mut()
//         // todo!()
//     }
// }
impl<T> tonic::codegen::tokio_stream::Stream for ThingStream4<T> {
    type Item = std::result::Result<T, tonic::Status>;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();

        let ff = this.fut.get_mut();
        let p = futures::FutureExt::poll_unpin(ff, cx);

        eprintln!("poll fut result: {p:?}");

        // let ts = self.get_mut();
        // let pinny = ts.thingy();

        let feat = (this.it)();
        std::task::Poll::Ready(feat)
    }
}


fn dist(pt1: routeguide::Point, pt2: routeguide::Point) -> i32 {
    let latd = pt1.latitude - pt2.latitude;
    let longd = pt1.longitude - pt2.longitude;
    let d = f64::sqrt((latd * latd + longd * longd) as f64);
    d as i32
}

struct AnotherNextinator {
    index: usize,
    pages: Vec<String>,
}

impl AnotherNextinator {
    fn new() -> Self {
        let pages = ["https://www.fermyon.com/blog/sqlite-is-edge-scale", "https://www.fermyon.com/blog/openapi-docs-for-spin-with-rust", "https://www.fermyon.com/blog/building-a-graphql-api-with-fwf"].iter().map(|s| s.to_string()).collect();
        Self { index: 0, pages }
    }

    fn next(&mut self) -> Option<Result<routeguide::Feature, tonic::Status>> {
        let index = self.index;

        let Some(page) = self.pages.get(index) else {
            return None;
        };
        self.index = index + 1;

        let req = spin_sdk::http::Request::get(page).build();
        let fut = spin_sdk::http::send::<spin_sdk::http::Request, spin_sdk::http::Response>(req);
        let resp = spin_executor::run(fut);
        Some(match resp {
            Ok(r) => Ok(routeguide::Feature { name: format!("{page} has content-len {:?}", r.header("content-length").and_then(|hv| hv.as_str())), location: None }),
            Err(e) => Err(tonic::Status::from_error(Box::new(e)))
        })
    }
}


// struct TheNextInator {
//     index: usize,
//     pages: Vec<String>,
// }

// impl TheNextInator {
//     fn new() -> Self {
//         let pages = ["https://www.fermyon.com/blog/sqlite-is-edge-scale", "https://www.fermyon.com/blog/openapi-docs-for-spin-with-rust", "https://www.fermyon.com/blog/building-a-graphql-api-with-fwf"].iter().map(|s| s.to_string()).collect();
//         Self { index: 0, pages }
//     }
// }

// // #[tonic::async_trait]
// impl AsyncNext for TheNextInator {
//     type Item = Result<routeguide::Feature, tonic::Status>;

//     #[must_use]
//     #[allow(elided_named_lifetimes,clippy::type_complexity,clippy::type_repetition_in_bounds)]
//     fn next<'life0,'async_trait>(&'life0 mut self) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Option<Self::Item> > + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,Self:'async_trait {
//         use futures::future::FutureExt;

//         let index = self.index;

//         let Some(page) = self.pages.get(index) else {
//             return Box::pin(core::future::ready(None));
//         };
//         self.index = index + 1;

//         let req = spin_sdk::http::Request::get(page).build();
//         let fut = spin_sdk::http::send::<spin_sdk::http::Request, spin_sdk::http::Response>(req)
//             // .map_err(|e| tonic::Status::from_error(Box::new(e)))
//             .map(|result| result
//                 .map(|r| routeguide::Feature { name: format!("{page} has content-len {:?}", r.header("content-length").and_then(|hv| hv.as_str())), location: None })
//                 .map_err(|e| tonic::Status::from_error(Box::new(e)))
//             )
//             .map(Some);

//         Box::pin(fut)

//             // .map_err(|e| tonic::Status::from_error(Box::new(e)))
//             // .map(|r| routeguide::Feature { name: format!("{page} has content-len {:?}", r.header("content-length").and_then(|hv| hv.as_str())), location: None });
//         // todo!()
//     }
    
//     // async fn next(&mut self) -> Option<Self::Item> {
//     //     let index = self.index;

//     //     let Some(page) = self.pages.get(index) else {
//     //         return None;
//     //     };
//     //     self.index = index + 1;

//     //     spin_sdk::http::send::<spin_sdk::http::Response>(spin_sdk::http::Request::get(page)).await
//     //         .map_err(|e| tonic::Status::from_error(Box::new(e)))
//     //         .map(|r| routeguide::Feature { name: format!("{page} has content-len {:?}", r.header("content-length").and_then(|hv| hv.as_str())), location: None })

//     //     let resp: Result<spin_sdk::http::Response, SendError> = spin_sdk::http::send(spin_sdk::http::Request::get(page)).await;

//     //     let item = match resp {
//     //         Ok(r) => Ok(routeguide::Feature { name: format!("{page} has content-len {:?}", r.header("content-length").and_then(|hv| hv.as_str())), location: None }),
//     //         Err(e) => Err(tonic::Status::from_error(Box::new(e))),
//     //     };

//     //     Some(item)

//     //     // if self.index > 4 {
//     //     //     None
//     //     // } else {
//     //     //     Some(Ok(routeguide::Feature { name: format!("Thing {}", self.index), location: None }))
//     //     // }
//     // }
// }

struct ThingStream<T> {
    it: Box<dyn (FnMut() -> Option<T>) + Send>,
}
struct ThingStream3<T> {
    it: Box<dyn (FnMut() -> Option<std::result::Result<T, tonic::Status>>) + Send>,
}

impl<T> tonic::codegen::tokio_stream::Stream for ThingStream<T> {
    type Item = std::result::Result<T, tonic::Status>;

    fn poll_next(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        let feat = (self.get_mut().it)().map(Ok);
        std::task::Poll::Ready(feat)
    }
}
impl<T> tonic::codegen::tokio_stream::Stream for ThingStream3<T> {
    type Item = std::result::Result<T, tonic::Status>;

    fn poll_next(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        let feat = (self.get_mut().it)();
        std::task::Poll::Ready(feat)
    }
}

#[tonic::async_trait]
trait AsyncNext {
    type Item;
    async fn next(&mut self) -> Option<Self::Item>;
}

struct ThingStream2<N> where N: AsyncNext {
    inner: N,
}

impl<N: AsyncNext + Unpin> tonic::codegen::tokio_stream::Stream for ThingStream2<N> {
    type Item = N::Item;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        use futures::Future;
        let s = self.get_mut();
        let fut = s.inner.next();
        futures::pin_mut!(fut);
        let poll = fut.as_mut().poll(cx);
        poll
    }
}
