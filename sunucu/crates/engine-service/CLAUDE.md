# engine-service

## Ne yapar
- `engine` kütüphanesini gRPC ile sunar: `EngineService`, `ToolService`, `RecipeService`, `MashService`, `WaterService`, `YeastService`, `FermentationService`, `BreweryService` (`demleme.engine.v1`, ADR-0005) ve standart sağlık servisi.
- Birim dönüşümü yalnız `convert.rs`'te; ortak girdi denetimi `check.rs`'te; uçtan uca test `tests/uc_uca.rs`.
- Sözleşme `sozlesme/proto/engine/v1/*.proto`; `build.rs` onu `protox` ile derler (sistemde protoc gerekmez).
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
