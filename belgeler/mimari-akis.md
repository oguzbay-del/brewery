# GRISTORA — Mimari akış

Bu şema, istemcilerden platform altyapısına kadar ana bileşenleri ve aralarındaki bağlantıları gösterir. Oklar çağrı veya veri akışının yönünü belirtir; çift yönlü ok yalnızca iki yönde iletişim olan bağlantılarda kullanılır.

```mermaid
flowchart TB
  subgraph clients["1 · İstemciler"]
    IOS["iPhone / iPad\nSwiftUI"]
    WEB["Web\nReact + TypeScript"]
  end

  subgraph edge["2 · Kenar"]
    EDGE["Edge\nTLS · WAF · hız sınırı · CDN"]
  end

  subgraph app["3 · Uygulama servisleri"]
    API["API\nRust"]
    WORKER["Worker\nRust"]
    INGEST["Ingest\nRust"]
  end

  subgraph calc["4 · Hesap"]
    ENGINE["Calculation engine\nRust · gRPC"]
  end

  subgraph data["5 · Veri"]
    PG[("PostgreSQL")]
    REDIS[("Redis")]
    NATS[("NATS")]
    OBJECTS[("S3 object store")]
  end

  subgraph external["Dış servisler ve cihazlar"]
    APPLE["Apple sign-in"]
    GOOGLE["Google sign-in"]
    STORE["App Store"]
    APNS["APNs"]
    EMAIL["Email provider"]
    GITHUB["GitHub"]
    DEVICES["iSpindel · RAPT"]
    PADDLE["Paddle"]
  end

  subgraph platform["6 · Platform"]
    OTEL["OpenTelemetry"]
    CLOUD["Cloud infrastructure"]
  end

  IOS -->|HTTPS / API| EDGE
  WEB -->|HTTPS / API| EDGE
  EDGE -->|Web delivery| WEB
  EDGE -->|Application requests| API
  EDGE -->|Inbound webhooks| INGEST

  API -->|gRPC calculations| ENGINE
  API -->|SQL| PG
  API -->|Cache| REDIS
  API -->|Events| NATS
  API -->|Signed file access| OBJECTS
  WORKER -->|SQL| PG
  WORKER -->|Cache| REDIS
  WORKER <-->|Jobs and events| NATS
  WORKER -->|File operations| OBJECTS
  INGEST -->|SQL| PG
  INGEST -->|Events| NATS

  API <-->|Identity verification| APPLE
  API <-->|Identity verification| GOOGLE
  STORE -->|Subscription notifications| INGEST
  WORKER -->|Push notifications| APNS
  WORKER -->|Transactional email| EMAIL
  GITHUB -->|Webhooks| INGEST
  WORKER <-->|Support issue sync| GITHUB
  DEVICES -->|Measurements and webhooks| INGEST
  API <-->|Checkout and payment events| PADDLE

  API -->|Traces and metrics| OTEL
  WORKER -->|Traces and metrics| OTEL
  INGEST -->|Traces and metrics| OTEL
  ENGINE -->|Traces and metrics| OTEL
  platform -->|Runs on| CLOUD

  classDef pending stroke-dasharray: 5 4,stroke-width:1.5px;
  class EDGE,API,WORKER,INGEST,ENGINE,PG,REDIS,NATS,OBJECTS,OTEL,CLOUD,IOS,WEB fill:#f6f8fa,stroke:#57606a,color:#24292f;
  class APPLE,GOOGLE,STORE,APNS,EMAIL,GITHUB,DEVICES,PADDLE fill:#fff8e1,stroke:#9a6700,color:#24292f;
```

## Kısa açıklama

- **İstemciler:** iPhone ve iPad yerel SwiftUI uygulamasını, web ise React + TypeScript arayüzünü kullanır. İkisi de aynı API'ye bağlanır. Hesaplama ve iş kuralları istemcide çalışmaz.
- **Kenar:** TLS, WAF ve hız sınırını uygular; web içeriğini CDN üzerinden sunar ve uygulama trafiğini API'ye yönlendirir.
- **Uygulama servisleri:** API iş kurallarını ve veri erişimini yürütür. Worker arka plan işlerini yapar. Ingest dış olayları ve cihaz verisini alır.
- **Hesap:** Rust hesap motoru ayrı bir iç servis olarak yalnız API tarafından gRPC ile çağrılır; doğrudan veriye erişmez.
- **Veri:** PostgreSQL kalıcı kayıtları, Redis önbelleği, NATS olay ve iş kuyruğunu, S3 uyumlu nesne deposu dosyaları tutar.
- **Platform:** Kubernetes üzerinde çalıştırma, bulut altyapısı ve OpenTelemetry gözlemlenebilirlik katmanını kapsar.

## Açık kalan altyapı işleri

Bu, mevcut ana dalın taslağıdır; tamamlanmamış altyapı işlerini tamamlanmış gibi göstermez:

- **Ağ politikaları:** #205 — PR #278 incelemede.
- **İç destek erişimi (oauth2-proxy):** #206 — açık.
- **mTLS:** #207 — açık.

T2.32'nin bu işleri tamamlandığında Claude son mimari kontrolü yapacak ve görsel Artifact sayfasını yayımlayacaktır.
