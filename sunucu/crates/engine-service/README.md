# engine-service

`engine` kütüphanesini gRPC ile sunar (`demleme.engine.v1.EngineService`) ve standart gRPC sağlık servisini ekler. Yalnız iç ağ içindir.
Sözleşme `sozlesme/proto/engine/v1/engine.proto`; `build.rs` onu saf Rust derleyici `protox` ile derler (sistemde `protoc` gerekmez).
Port `PORT` ortam değişkeninden (varsayılan 50051). Hesap kodu içermez, `engine`'i çağırır.
Test: `cargo test -p engine-service` (`sunucu/` içinde).
