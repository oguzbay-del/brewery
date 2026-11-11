# engine-service

## Ne yapar
- `engine` kütüphanesini gRPC ile sunar: `demleme.engine.v1.EngineService` ve standart sağlık servisi.
- Sözleşme `sozlesme/proto/engine/v1/engine.proto`; `build.rs` onu `protox` ile derler (sistemde protoc gerekmez).
- Port `PORT` (varsayılan 50051).

## Neye dokunmaz
- Yalnız iç ağ içindir; dışarıya açılmaz.
- Hesap kodu yazılmaz; hesap `engine`'de, bu crate onu çağırır.
- Proto sözleşmesi `sozlesme/` altında değişir, bu crate'te değil.

## Nasıl test edilir
- `sunucu/` içinde: `cargo test -p engine-service`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`

## Kurallar
- Proto değişince önce sözleşme güncellenir, sonra servis derlenir.
- Kod İngilizce; belgeler Türkçe.
- Commit mesajı iş numarasıyla başlar: `T0.x: ...`.
