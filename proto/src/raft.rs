#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LogEntry {
    #[prost(uint64, tag="1")]
    pub term: u64,
    #[prost(bytes="vec", tag="2")]
    pub command: ::prost::alloc::vec::Vec<u8>,
}
/// Invoked by leader to replicate log entries.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AppendEntriesRequest {
    /// The election term that the leader node is in.
    #[prost(uint64, tag="1")]
    pub term: u64,
    /// The ID of the leader so that follower can redirect clients.
    #[prost(uint64, tag="2")]
    pub leader_id: u64,
    /// The index of the log entry immediately preceding new ones.
    #[prost(uint64, tag="3")]
    pub prev_log_index: u64,
    /// The term of the prev_log_index.
    #[prost(uint64, tag="4")]
    pub prev_log_term: u64,
    /// The log entries to store.
    #[prost(message, repeated, tag="5")]
    pub entries: ::prost::alloc::vec::Vec<LogEntry>,
    /// The leader's commit index.
    #[prost(uint64, tag="6")]
    pub leader_commit_index: u64,
}
/// Returned by candidates and followers when a leader requests to AppendEntries.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AppendEntriesResponse {
    /// The current term for leader to update itself.
    #[prost(uint64, tag="1")]
    pub term: u64,
    /// True, if a follower contained an entry matching prev_log_index and prev_log_term
    #[prost(bool, tag="2")]
    pub success: bool,
    /// The term for the earliest conflicting entry, if unsuccessful.
    #[prost(uint64, tag="3")]
    pub conflicting_term: u64,
    /// The index of the first entry in the conflicting term, if unsuccessful.
    #[prost(uint64, tag="4")]
    pub conflicting_term_first_index: u64,
}
/// Invoked by leader to inform the other servers of its leadership and existence.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeartbeatRequest {
    /// The election term that the leader node is in.
    #[prost(uint64, tag="1")]
    pub term: u64,
    /// The ID of the leader so that follower can redirect clients.
    #[prost(uint64, tag="2")]
    pub leader_id: u64,
    /// The index of the log entry immediately preceding new ones.
    #[prost(uint64, tag="3")]
    pub prev_log_index: u64,
    /// The term of the prev_log_index.
    #[prost(uint64, tag="4")]
    pub prev_log_term: u64,
    /// The leader's commit index.
    #[prost(uint64, tag="5")]
    pub leader_commit_index: u64,
}
/// Returned by candidates and followers when a leader tells them of its presence.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeartbeatResponse {
    /// The current term for leader to update itself.
    #[prost(uint64, tag="1")]
    pub term: u64,
    /// True, if a follower contained an entry matching prev_log_index and prev_log_term
    #[prost(bool, tag="2")]
    pub success: bool,
}
/// Invoked by candidate to gather votes.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VoteRequest {
    /// The election term that the candidate node is in.
    #[prost(uint64, tag="1")]
    pub term: u64,
    /// The ID of the candidate node that is requesting a vote.
    #[prost(uint64, tag="2")]
    pub candidate_id: u64,
    /// The index of the candidate's last log entry.
    #[prost(uint64, tag="3")]
    pub last_log_index: u64,
    /// The term of the candidate's last log entry.
    #[prost(uint64, tag="4")]
    pub last_log_term: u64,
}
/// Returned by followers when a candidate requests for vote.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VoteResponse {
    /// The election term that the follower node is in.
    #[prost(uint64, tag="1")]
    pub term: u64,
    /// Whether the follower node granted a vote to this candidate.
    #[prost(bool, tag="2")]
    pub vote_granted: bool,
}
/// Generated client implementations.
pub mod raft_rpc_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct RaftRpcClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl RaftRpcClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> RaftRpcClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> RaftRpcClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            RaftRpcClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        /// Enable decompressing responses with `gzip`.
        #[must_use]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        /// A leader-to-* RPC that requests the other nodes to append entries in their logs.
        pub async fn append_entries(
            &mut self,
            request: impl tonic::IntoRequest<super::AppendEntriesRequest>,
        ) -> Result<tonic::Response<super::AppendEntriesResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/raft.RaftRpc/append_entries",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// A leader-to-* RPC that informs the other notes of its leadership and existence.
        pub async fn heartbeat(
            &mut self,
            request: impl tonic::IntoRequest<super::HeartbeatRequest>,
        ) -> Result<tonic::Response<super::HeartbeatResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/raft.RaftRpc/Heartbeat");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// A candidate-to-follower RPC that requests for a vote to become leader in the latest election term.
        pub async fn request_vote(
            &mut self,
            request: impl tonic::IntoRequest<super::VoteRequest>,
        ) -> Result<tonic::Response<super::VoteResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/raft.RaftRpc/RequestVote");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod raft_rpc_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with RaftRpcServer.
    #[async_trait]
    pub trait RaftRpc: Send + Sync + 'static {
        /// A leader-to-* RPC that requests the other nodes to append entries in their logs.
        async fn append_entries(
            &self,
            request: tonic::Request<super::AppendEntriesRequest>,
        ) -> Result<tonic::Response<super::AppendEntriesResponse>, tonic::Status>;
        /// A leader-to-* RPC that informs the other notes of its leadership and existence.
        async fn heartbeat(
            &self,
            request: tonic::Request<super::HeartbeatRequest>,
        ) -> Result<tonic::Response<super::HeartbeatResponse>, tonic::Status>;
        /// A candidate-to-follower RPC that requests for a vote to become leader in the latest election term.
        async fn request_vote(
            &self,
            request: tonic::Request<super::VoteRequest>,
        ) -> Result<tonic::Response<super::VoteResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct RaftRpcServer<T: RaftRpc> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: RaftRpc> RaftRpcServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for RaftRpcServer<T>
    where
        T: RaftRpc,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/raft.RaftRpc/append_entries" => {
                    #[allow(non_camel_case_types)]
                    struct append_entriesSvc<T: RaftRpc>(pub Arc<T>);
                    impl<
                        T: RaftRpc,
                    > tonic::server::UnaryService<super::AppendEntriesRequest>
                    for append_entriesSvc<T> {
                        type Response = super::AppendEntriesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AppendEntriesRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).append_entries(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = append_entriesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/raft.RaftRpc/Heartbeat" => {
                    #[allow(non_camel_case_types)]
                    struct HeartbeatSvc<T: RaftRpc>(pub Arc<T>);
                    impl<T: RaftRpc> tonic::server::UnaryService<super::HeartbeatRequest>
                    for HeartbeatSvc<T> {
                        type Response = super::HeartbeatResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::HeartbeatRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).heartbeat(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = HeartbeatSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/raft.RaftRpc/RequestVote" => {
                    #[allow(non_camel_case_types)]
                    struct RequestVoteSvc<T: RaftRpc>(pub Arc<T>);
                    impl<T: RaftRpc> tonic::server::UnaryService<super::VoteRequest>
                    for RequestVoteSvc<T> {
                        type Response = super::VoteResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::VoteRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).request_vote(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RequestVoteSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: RaftRpc> Clone for RaftRpcServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: RaftRpc> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: RaftRpc> tonic::transport::NamedService for RaftRpcServer<T> {
        const NAME: &'static str = "raft.RaftRpc";
    }
}
