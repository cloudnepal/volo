use std::net::SocketAddr;

use volo_grpc::{
    layer::grpc_web::GrpcWebLayer,
    server::{Server, ServiceBuilder},
};

pub struct S;

impl volo_gen::proto_gen::helloworld::Greeter for S {
    async fn say_hello(
        &self,
        req: volo_grpc::Request<volo_gen::proto_gen::helloworld::HelloRequest>,
    ) -> Result<volo_grpc::Response<volo_gen::proto_gen::helloworld::HelloReply>, volo_grpc::Status>
    {
        let resp = volo_gen::proto_gen::helloworld::HelloReply {
            message: format!("helloworld, {}!", req.get_ref().name).into(),
        };
        Ok(volo_grpc::Response::new(resp))
    }
}

#[volo::main]
async fn main() {
    let addr: SocketAddr = "[::]:8080".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    Server::new()
        .accept_http1(true)
        .layer_tower(GrpcWebLayer::new())
        .add_service(
            ServiceBuilder::new(volo_gen::proto_gen::helloworld::GreeterServer::new(S)).build(),
        )
        .run(addr)
        .await
        .unwrap();
}
