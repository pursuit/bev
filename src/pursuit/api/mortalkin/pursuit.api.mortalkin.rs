#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlayGamePayload {
    #[prost(string, tag = "1")]
    pub token: ::prost::alloc::string::String,
    #[prost(int64, tag = "2")]
    pub character_id: i64,
    #[prost(message, optional, tag = "3")]
    pub position: ::core::option::Option<Position>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GameNotif {
    #[prost(message, repeated, tag = "1")]
    pub character_on_notifs: ::prost::alloc::vec::Vec<Character>,
    #[prost(message, repeated, tag = "2")]
    pub character_position_notifs: ::prost::alloc::vec::Vec<CharacterPosition>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Character {
    #[prost(int64, tag = "1")]
    pub id: i64,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub position: ::core::option::Option<Position>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CharacterPosition {
    #[prost(int64, tag = "1")]
    pub character_id: i64,
    #[prost(message, optional, tag = "2")]
    pub position: ::core::option::Option<Position>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Position {
    #[prost(int32, tag = "1")]
    pub x: i32,
    #[prost(int32, tag = "2")]
    pub y: i32,
}
#[doc = r" Generated client implementations."]
pub mod game_client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    pub struct GameClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl GameClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> GameClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + HttpBody + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as HttpBody>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
            let inner = tonic::client::Grpc::with_interceptor(inner, interceptor);
            Self { inner }
        }
        pub async fn play(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::PlayGamePayload>,
        ) -> Result<tonic::Response<tonic::codec::Streaming<super::GameNotif>>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/pursuit.api.mortalkin.Game/Play");
            self.inner
                .streaming(request.into_streaming_request(), path, codec)
                .await
        }
    }
    impl<T: Clone> Clone for GameClient<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
    impl<T> std::fmt::Debug for GameClient<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "GameClient {{ ... }}")
        }
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LoginPayload {
    #[prost(string, tag = "1")]
    pub username: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub password: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LoginResponse {
    #[prost(string, tag = "1")]
    pub token: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub characters: ::prost::alloc::vec::Vec<Character>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateCharacterPayload {
    #[prost(string, tag = "1")]
    pub token: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
}
#[doc = r" Generated client implementations."]
pub mod user_client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    pub struct UserClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl UserClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> UserClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + HttpBody + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as HttpBody>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
            let inner = tonic::client::Grpc::with_interceptor(inner, interceptor);
            Self { inner }
        }
        pub async fn login(
            &mut self,
            request: impl tonic::IntoRequest<super::LoginPayload>,
        ) -> Result<tonic::Response<super::LoginResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/pursuit.api.mortalkin.User/Login");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create_character(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateCharacterPayload>,
        ) -> Result<tonic::Response<super::Character>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/pursuit.api.mortalkin.User/CreateCharacter");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
    impl<T: Clone> Clone for UserClient<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
    impl<T> std::fmt::Debug for UserClient<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "UserClient {{ ... }}")
        }
    }
}
