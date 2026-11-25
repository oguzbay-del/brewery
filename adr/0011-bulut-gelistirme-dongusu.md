# ADR-0011: Bulut geliştirme döngüsü v2 (Claude ↔ ChatGPT/Codex)

**Durum:** kabul (pilot) · **Tarih:** 25 Kasım 2026 · **Karar:** Oğuz (25.11.2026) · **İş:** döngü v2

## Bağlam

22.11.2026'da ajan köprüsü (Cloudflare Worker, runner'lar, OpenAI API) maliyet yüzünden iptal edildi. O tarihten beri Claude ve ChatGPT GitHub etiketleri üzerinden çalışıyor, ama tetikleme elle ("Devam") ya da Claude'un zamanlanmış görevleriyle yapılıyor. 25.11.2026'da yapılan denetimde Claude'un iki saatlik "ChatGPT iş atama" görevinin depoya bağlı olmadığı, her çalışmada GitHub 403 alıp hiçbir şey yapmadığı görüldü; altı ChatGPT ürün PR'ı inceleme bekliyordu.

Owner kararı: sistem tamamen bulutta çalışacak; owner'ın bilgisayarı kapalıyken görev atama, geliştirme, test, review, geri bildirim ve düzeltme döngüsü sürecek. Mevcut ücretli aboneliklerin yerel bulut özellikleri önceliklidir; ek maliyet owner onayı ister; köprü, MCP sunucusu, Worker ve Vercel kurulmaz.

## Doğrulanan yetenekler (25.11.2026)

- **Claude Code zamanlanmış görevleri** (eski adı routines): bulutta çalışır; tetikleyiciler zamanlama (en sık saatlik), API (`/fire`, bearer token) ve GitHub olaylarıdır (pull request, issue, release). GitHub tetikleyicisi Claude GitHub App ister ve web arayüzünden eklenir; olay başına saatlik üst sınır vardır. PR yorumu ve CI sonucu olay listesinde yok. Kaynak: code.claude.com/docs/en/web-scheduled-tasks.
- **Claude oturumu PR aboneliği:** depoya bağlı uzun ömürlü bir oturum, abone olduğu PR'ın yorum, review ve CI olaylarını alır.
- **Ajanın oluşturduğu zamanlanmış görev** depo bağlamaz; görevi depoya bağlı bir oturuma yönlendirmek (persistent session) gerekir.
- **Codex:** PR yorumunda `@codex review` ve görev cümlesiyle `@codex ...` bulut görevi başlatır; otomatik review ayarı vardır, yalnız P0/P1 bulguları yazar. Kaynak: learn.chatgpt.com/docs/third-party/github. Issue yorumunda ve bizim hesap düzenimizde (Claude ve ChatGPT aynı GitHub hesabıyla yazıyor) tetiklenip tetiklenmediği pilotta denenir.
- **ChatGPT zamanlanmış görevleri:** web görevleri bulutta çalışır; uygun planlarda GitHub PR etkinliği (review, yorum, commit) tetikleyicisi vardır. Kurulum yalnız owner'ın ChatGPT hesabından yapılır. Kaynak: learn.chatgpt.com/docs/automations.
- **GitHub:** depo herkese açık; Actions dakikaları ücretsiz.

## Karar

1. Ortak kuyruk, durum ve kayıt GitHub'dır (etiketler, issue, PR, CI). Kurallar `gelistirme/dongu/README.md`'de; Codex için kök `AGENTS.md`.
2. Claude tarafı: depoya bağlı tek bir "döngü oturumu"; saatlik zamanlanmış görev (`:17`) bu oturuma düşer (watchdog). Oturum incelediği PR'lara abone olur; yorum, review ve CI olayları anında gelir. İsteğe bağlı GitHub olay tetikleyicisini owner web arayüzünden ekler.
3. ChatGPT tarafı: owner'ın kuracağı saatlik ChatGPT görevi ve PR etkinliği görevi; Codex otomatik review'ı ve `@codex` yorumları.
4. Claude → ChatGPT olayı: issue/PR yorumunda `@codex`. ChatGPT → Claude olayı: PR ve PR etkinliği (abonelik), yedeği saatlik watchdog.
5. Kimlik aynı GitHub hesabı olduğu için her otomatik yorum işaretçiyle başlar (`<!-- loop:v2 actor=... kind=... ref=<sha> round=n -->`). Aynı head SHA için aynı işlem bir kez yapılır; PR başına en çok 3 düzeltme turu, sonra `loop:needs-owner`. `LOOP-CONTROL` issue'sundaki `loop:pause` bütün döngüyü durdurur.
6. Merge, production ve güvenlik kuralları değişmez. Codex merge etmez. Claude'un C sınıfı PR'ı, açık P0/P1 bırakmayan bir Codex review'ı ya da owner onayı olmadan merge edilmez.
7. Çalışmayan eski iki "ChatGPT iş atama" görevi kapatılır; döngü oturumu yerini alır.

## Sonuçlar

- Owner'ın bilgisayarına bağımlılık yok. Ek maliyet yok (abonelik kullanımı ve herkese açık depo için ücretsiz Actions).
- Owner'ın bir kez yapacağı adımlar: ChatGPT'de iki görev, Codex ayarları, isteğe bağlı Claude GitHub tetikleyicisi (`gelistirme/dongu/README.md` › 9).
- Risk: iki ajanın aynı hesabı kullanması GitHub'da yazarı ayırt etmeyi engeller; işaretçi ve dal öneki bunu karşılar. Codex'in, Claude'un yazdığı `@codex` yorumuna yanıt verip vermediği pilotla doğrulanır; vermezse ChatGPT'nin saatlik görevi yedektir.
- Risk: döngü oturumunun bağlamı büyür; otomatik sıkıştırma ve her çalışmada kuralların yeniden okunması bunu sınırlar. Oturum bozulursa yeni bir oturum açılıp saatlik görev ona yönlendirilir.
- 22.11 kararının "otomatik review yok" kısmının yerini bu karar alır; köprü ve API anahtarı yasağı sürer.
