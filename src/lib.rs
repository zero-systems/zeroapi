pub mod impl_blocking {
    pub trait Request<C, Rs> {
        fn execute(self, client: C) -> Rs;
    }

    pub trait Context<C, Rs, Rq>
    where
        Rq: Request<C, Rs>,
    {
        fn commit(self, request: Rq) -> Rs;
    }

    impl<C, Rs, Rq> Context<C, Rs, Rq> for C
    where
        Rq: Request<C, Rs>,
    {
        fn commit(self, request: Rq) -> Rs {
            request.execute(self)
        }
    }

    #[test]
    fn test_move() {
        use crate::impl_blocking::{Context, Request};

        pub struct ClientS {
            pub num: u32,
        };
        pub struct RequestS {
            pub num: u32,
        };
        pub struct ResponseS {
            pub num: u32,
        };

        impl Request<ClientS, ResponseS> for RequestS {
            fn execute(self, client: ClientS) -> ResponseS {
                ResponseS {
                    num: client.num + self.num,
                }
            }
        }

        let request = RequestS { num: 2 };
        let result = ClientS { num: 1 }.commit(request);

        assert_eq!(result.num, 3);
    }

    #[test]
    fn test_req_ref() {
        use crate::impl_blocking::{Context, Request};

        pub struct ClientS {
            pub num: u32,
        };
        pub struct RequestS {
            pub num: u32,
        };
        pub struct ResponseS {
            pub num: u32,
        };

        impl Request<ClientS, ResponseS> for &RequestS {
            fn execute(self, client: ClientS) -> ResponseS {
                ResponseS {
                    num: client.num + self.num,
                }
            }
        }

        let request = RequestS { num: 2 };
        let result = ClientS { num: 1 }.commit(&request);

        assert_eq!(result.num, 3);
    }

    #[test]
    fn test_mut() {
        use crate::impl_blocking::{Context, Request};

        pub struct ClientS {
            pub num: u32,
        };
        pub struct RequestS {
            pub num: u32,
        };
        pub struct ResponseS {
            pub num: u32,
        };

        impl Request<ClientS, ResponseS> for &mut RequestS {
            fn execute(self, client: ClientS) -> ResponseS {
                self.num = 5;
                ResponseS {
                    num: client.num + self.num,
                }
            }
        }

        let client = ClientS { num: 1 };
        let mut request = RequestS { num: 2 };
        let result = client.commit(&mut request);

        assert_eq!(result.num, 6);
        assert_eq!(request.num, 5);

        // var still exists!
        let _ = client;
    }

    #[test]
    fn test_client_ref() {
        use crate::impl_blocking::{Context, Request};

        pub struct ClientS {
            pub num: u32,
        };
        pub struct RequestS {
            pub num: u32,
        };
        pub struct ResponseS {
            pub num: u32,
        };

        impl Request<&ClientS, ResponseS> for RequestS {
            fn execute(self, client: &ClientS) -> ResponseS {
                ResponseS {
                    num: client.num + self.num,
                }
            }
        }

        let client = ClientS { num: 1 };
        let request = RequestS { num: 2 };
        let result = (&client).commit(request);

        assert_eq!(result.num, 3);

        // var still exists!
        let _ = client;
    }

    #[test]
    fn test_client_mut() {
        use crate::impl_blocking::{Context, Request};

        pub struct ClientS {
            pub num: u32,
        };
        pub struct RequestS {
            pub num: u32,
        };
        pub struct ResponseS {
            pub num: u32,
        };

        impl Request<&mut ClientS, ResponseS> for RequestS {
            fn execute(self, client: &mut ClientS) -> ResponseS {
                client.num = 5;
                ResponseS {
                    num: client.num + self.num,
                }
            }
        }

        let mut client = ClientS { num: 1 };
        let request = RequestS { num: 2 };
        let result = (&mut client).commit(request);

        assert_eq!(result.num, 7);
        assert_eq!(client.num, 5);

        // var still exists!
        let _ = client;
    }
}

pub mod impl_async {
    use std::{future::Future, pin::Pin};

    pub trait RequestAsync<'f, C, Rs> {
        fn execute_async(self, client: C) -> Pin<Box<dyn Future<Output = Rs> + 'f>>
        where
            Self: 'f;
    }

    pub trait ContextAsync<'f, C, Rs, Rq>
    where
        Self: 'f,
        Rq: RequestAsync<'f, C, Rs> + 'f,
    {
        fn commit_async(self, request: Rq) -> Pin<Box<dyn Future<Output = Rs> + 'f>>;
    }

    impl<'f, C, Rs, Rq> ContextAsync<'f, C, Rs, Rq> for C
    where
        Self: 'f,
        Rq: RequestAsync<'f, C, Rs> + 'f,
    {
        fn commit_async(self, request: Rq) -> Pin<Box<dyn Future<Output = Rs> + 'f>> {
            request.execute_async(self)
        }
    }

    #[test]
    fn test_move() {
        use crate::impl_async::{ContextAsync, RequestAsync};
        use futures::executor::block_on;
        use std::{future::Future, pin::Pin};

        pub struct ClientS {
            pub num: u32,
        };
        pub struct RequestS {
            pub num: u32,
        };
        pub struct ResponseS {
            pub num: u32,
        };

        impl RequestAsync<'_, ClientS, ResponseS> for RequestS {
            fn execute_async(self, client: ClientS) -> Pin<Box<dyn Future<Output = ResponseS>>> {
                Box::pin(async move {
                    ResponseS {
                        num: client.num + self.num,
                    }
                })
            }
        }

        let request = RequestS { num: 2 };
        let future = ClientS { num: 1 }.commit_async(request);
        let result = block_on(future);

        assert_eq!(result.num, 3);
    }

    #[test]
    fn test_req_ref() {
        use crate::impl_async::{ContextAsync, RequestAsync};
        use futures::executor::block_on;
        use std::{future::Future, pin::Pin};

        pub struct ClientS {
            pub num: u32,
        };
        pub struct RequestS {
            pub num: u32,
        };
        pub struct ResponseS {
            pub num: u32,
        };

        impl<'f> RequestAsync<'f, ClientS, ResponseS> for &'f RequestS {
            fn execute_async(
                self,
                client: ClientS,
            ) -> Pin<Box<dyn Future<Output = ResponseS> + 'f>> {
                Box::pin(async move {
                    ResponseS {
                        num: client.num + self.num,
                    }
                })
            }
        }

        let client = ClientS { num: 1 };
        let request = RequestS { num: 2 };
        let future = client.commit_async(&request);
        let result = block_on(future);

        assert_eq!(result.num, 3);

        // var still exists!
        let _ = client;
    }

    #[test]
    fn test_mut() {
        use crate::impl_async::{ContextAsync, RequestAsync};
        use futures::executor::block_on;
        use std::{future::Future, pin::Pin};

        pub struct ClientS {
            pub num: u32,
        };
        pub struct RequestS {
            pub num: u32,
        };
        pub struct ResponseS {
            pub num: u32,
        };

        impl<'f> RequestAsync<'f, ClientS, ResponseS> for &'f mut RequestS {
            fn execute_async(
                self,
                client: ClientS,
            ) -> Pin<Box<dyn Future<Output = ResponseS> + 'f>> {
                Box::pin(async move {
                    self.num = 5;
                    ResponseS {
                        num: client.num + self.num,
                    }
                })
            }
        }

        let client = ClientS { num: 1 };
        let mut request = RequestS { num: 2 };
        let future = client.commit_async(&mut request);
        let result = block_on(future);

        assert_eq!(result.num, 6);
        assert_eq!(request.num, 5);

        // var still exists!
        let _ = client;
    }

    #[test]
    fn test_client_ref() {
        use crate::impl_async::{ContextAsync, RequestAsync};
        use futures::executor::block_on;
        use std::{future::Future, pin::Pin};

        pub struct ClientS {
            pub num: u32,
        };
        pub struct RequestS {
            pub num: u32,
        };
        pub struct ResponseS {
            pub num: u32,
        };

        impl<'f> RequestAsync<'f, &'f ClientS, ResponseS> for RequestS {
            fn execute_async(
                self,
                client: &'f ClientS,
            ) -> Pin<Box<dyn Future<Output = ResponseS> + 'f>> {
                Box::pin(async move {
                    ResponseS {
                        num: client.num + self.num,
                    }
                })
            }
        }

        let client = ClientS { num: 1 };
        let request = RequestS { num: 2 };
        let future = (&client).commit_async(request);
        let result = block_on(future);

        assert_eq!(result.num, 3);

        // var still exists!
        let _ = client;
    }

    #[test]
    fn test_client_mut() {
        use crate::impl_async::{ContextAsync, RequestAsync};
        use futures::executor::block_on;
        use std::{future::Future, pin::Pin};

        pub struct ClientS {
            pub num: u32,
        };
        pub struct RequestS {
            pub num: u32,
        };
        pub struct ResponseS {
            pub num: u32,
        };

        impl<'f> RequestAsync<'f, &'f mut ClientS, ResponseS> for RequestS {
            fn execute_async(
                self,
                client: &'f mut ClientS,
            ) -> Pin<Box<dyn Future<Output = ResponseS> + 'f>> {
                Box::pin(async move {
                    client.num = 5;
                    ResponseS {
                        num: client.num + self.num,
                    }
                })
            }
        }

        let mut client = ClientS { num: 1 };
        let request = RequestS { num: 2 };
        let future = (&mut client).commit_async(request);
        let result = block_on(future);

        assert_eq!(result.num, 7);
        assert_eq!(client.num, 5);

        // var still exists!
        let _ = client;
    }
}
