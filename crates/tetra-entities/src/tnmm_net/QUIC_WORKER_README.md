# Architecture

```
┌─────────────────────────────────────────┐
│   TETRA Stack Main Thread               │
│   (NetEntity component at Tnmm SAP)     │
└──────────────┬──────────────────────────┘
               │ crossbeam channels, 
               │   offload to separate thread
┌──────────────▼───────────────────────────────┐
│  Worker Thread                               │
│  (NetEntityTnmmWorkerQuic)                   │
│  ┌───────────────────────────────────────┐   │
│  │ Tokio Runtime (thread) per connection │   │
│  │ ┌─────────────────────────────────┐   │   │
│  │ │  QuicTransport                  │   │   │
│  │ │  - Reliable stream              │   │   │
│  │ │  - Optional: unreliable streams │   │   │
│  │ │  - TLS 1.3 encryption           │   │   │
│  │ └─────────────────────────────────┘   │   │
│  └───────────────────────────────────────┘   │
└──────────────┬───────────────────────────────┘
               │ QUIC over UDP
               │
┌──────────────▼──────────────────────────┐
│   Remote service, QUIC Server           │
│   (bins/net-tnmm-test-quic)             │
└─────────────────────────────────────────┘
```