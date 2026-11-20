import type { FeatureAudience, FeaturePage, FeatureStage } from "./features";

export const audienceLabelTr: Record<FeatureAudience, string> = {
  Breweries: "Bira üreticileri",
  Homebrewers: "Ev biracıları",
};

export const stageLabelTr: Record<FeatureStage, string> = {
  "Homebrew MVP": "Ev biracılığı ilk sürüm",
  "Homebrew second release": "Ev biracılığı ikinci sürüm",
  "Business core": "İşletme çekirdeği",
  "Business second release": "İşletme ikinci sürüm",
  Later: "Daha sonra",
  "Under consideration": "Değerlendirmede",
};

export const featuresTr: FeaturePage[] = [
  {
    slug: "recipe-design",
    modules: ["core"],
    title: "Reçete tasarımı ve yönetimi",
    eyebrow: "REÇETELER",
    summary: "Rehberli ilk reçeteden derinlemesine yapılandırılabilen profesyonel formülasyona kadar GRISTORA, reçeteleri sürümden sürüme anlaşılır tutar.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business core"],
    sections: [
      { heading: "Size uygun yoldan başlayın", intro: "Dört giriş noktası tek bir tutarlı reçete akışına çıkar.", items: ["Stil ve hedef odaklı malzeme önerileriyle boş bir reçeteden başlayın", "Stil tabanlı başlangıç şablonlarını kullanın ve ekipmanınıza ve parti hacminize uyarlayın", "Mevcut bir reçeteyi kopyalayın ya da bilinçli olarak yeni bir sürüm oluşturun", "BeerXML, BeerJSON, BeerSmith .bsmx, Brewfather ve Brewer's Friend dışa aktarımlarını; malzeme, ekipman, stil ve birim eşleştirmesiyle içe aktarın"] },
      { heading: "Yinelemeye uygun bir reçete düzenleyicisi", intro: "Düzenleyici, değişiklikleri ve sonuçlarını görünür kılmak için tasarlanmıştır.", items: ["Fermente edilebilir malzemeleri, şerbetçiotunu, mayayı, çeşitli malzemeleri, mayşeyi, kaynatmayı ve fermantasyon ayrıntılarını düzenleyin", "Seçili değerleri dengede tutarak tahıl yüzdelerini ve hedef OG, IBU ya da rengi ayarlayın", "Tam tahıl reçetelerini ekstrakta, ekstrakt reçetelerini tekrar tam tahıla çevirin; malzemeleri ikame edin ve miktarları yuvarlayın", "Kayıpları yeniden hesaplayarak parti hacmine, demhane verimine ya da ekipmana göre ölçekleyin", "Reçete kontrollerini, uyarıları ve tek adımlık düzeltme önerilerini inceleyin"] },
      { heading: "Stiller, sürümler ve iş birliği", intro: "Bir reçetenin arkasındaki referansı ve geçmişi net tutun.", items: ["BJCP 2021, BJCP bal likörü/elma şarabı ve Brewers Association stil rehberleri; özel rehber desteğiyle", "Stil aralıklarını karşılaştırın ve bir reçeteyi rehberler arasında eşleştirin", "İki reçeteyi ya da sürümü yan yana karşılaştırın", "Yayımlanmış sürümleri değiştirilemez biçimde saklayın; işletme reçeteleri taslak, inceleme ve onay aşamalarından geçer", "Reçeteyi yanlışlıkla yapılan düzenlemelere karşı kilitleyin ve her partinin kullandığı sürümü koruyun"] },
      { heading: "Reçete kütüphanesi ve desteklenen üretim türleri", intro: "Büyüyen bir reçete koleksiyonunu düzenli ve kullanışlı tutun.", items: ["Reçeteleri ve partileri arama, filtre, klasör ve etiketlerle düzenleyin", "Favorileri sabitleyin ve reçete listelerinde hangi sütunların görüneceğini seçin", "Tam tahıl, ekstrakt, kısmi mayşe ve BIAB yaklaşımları dahil bira, bal likörü, elma şarabı ve seltzer reçetelerini destekleyin", "Reçete notlarını, renk seçimlerini, görselleri ve özel alanları saklayın", "Yayımlanmış sürümlerin üzerine sessizce yazmadan reçete geçmişini arşivleyin ve geri yükleyin"] }
    ]
  },
  {
    slug: "brewing-calculations",
    modules: ["core"],
    title: "Bira hesaplamaları",
    eyebrow: "HESAP MOTORU",
    summary: "Ortak ve sunucu tarafında çalışan bir hesap motoru web ve Apple uygulamalarına hizmet eder; girdiler nettir, birimler tutarlıdır ve her sonucun arkasında bir açıklama bulunur.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business core", "Business second release", "Under consideration"],
    sections: [
      { heading: "Temel reçete hesaplamaları", intro: "Geniş bir hesaplama seti formülasyonu ve incelemeyi destekler.", items: ["Başlangıç ve son yoğunluk (OG / FG), görünür attenuasyon ve hacimce alkol (ABV)", "Varsayılan olarak Tinseth ile acılık (IBU); geç eklemeler, whirlpool ve hop-stand davranışı dahil", "Seçilebilir hesaplama yaklaşımıyla renk tahminleri", "Fermantasyon tahminlerinde mayşe sıcaklığı ve maya toleransı düzeltmeleri", "Parti, kazan, tortu ve hacim zinciri hesaplamaları"] },
      { heading: "Gelişmiş üretim modelleri", intro: "Motor, uzmanlık yöntemlerini ve yapılandırılabilir varsayımları içerir.", items: ["Sağlanan girdilerle şerbetçiotu yaşlanması ve alfa asit düzeltmeleri", "Starter büyümesi ve maya aşılama hesaplamaları", "Şişeler ve kegler için karbonasyon ve priming; keg üst boşluğu ve basınç sınırları dahil", "Düzeltilmiş YAN yönetimiyle bal likörü ve şarap fermantasyonu hesaplamaları", "Ekstrakt, kısmi kaynatma, seyreltme ve no-chill hesaplama araçları"] },
      { heading: "Açıklanabilir sonuçlar", intro: "Teknik şeffaflık bir ürün ilkesidir.", items: ["Temel sonuçlar için “Nasıl hesaplandı” görünümü", "Kaydedilen her sonuçla birlikte saklanan hesap motoru sürümü", "Bölgesel gösterim ve birim dönüşümüyle tutarlı, SI tabanlı iç değerler", "Her motor sürümü için referans test vektörleri ve altın reçete doğrulaması"] },
      { heading: "Uzman yöntemler", intro: "Ek yöntemler temel hesaplama setinden ayrı durur.", items: ["No-chill IBU, şerbetçiotu yağı oranları, BU:GU ile alternatif IBU ve renk yöntemleri", "Elma şarabı ve şarap için meyve eklemeleri, pH sıcaklık düzeltmesi ve toplam asitlik ayarı", "Ekşi ve karma fermantasyon hedefleri ile fıçıda olgunlaştırma hesaplamaları", "Hortum çapı, uzunluğu ve servis basıncı için keg hat dengesi araçları", "Hesaplama ayarlarında yöntem seçimi ve gelişmiş varsayılanlar"] }
    ],
  },
  {
    slug: "water-mash-yeast",
    modules: ["core"],
    title: "Su, mayşe ve maya",
    eyebrow: "SÜREÇ TASARIMI",
    summary: "Su kimyasını, mayşe adımlarını ve maya hazırlığını birbirinden bağımsız hesaplayıcılar olarak değil, reçetenin bağlantılı parçaları olarak planlayın.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Under consideration"],
    sections: [
      { heading: "Su profilleri", intro: "Kaynak sudan ve hedef stil ya da reçeteden bir su profili oluşturun.", items: ["Kaynak su ve hedef su profillerini kaydedin", "Laboratuvar su raporunu girin ve iyon konsantrasyonlarını yönetin", "Tuz eklemelerini ve asit ayarlarını hesaplayın", "İyon dengesini, sertliği ve bölgesel sertlik birimlerini inceleyin", "Su profillerini karşılaştırın ve bir reçetede kullanılan raporu saklayın"] },
      { heading: "Mayşe planlaması", intro: "Mayşe planı malzeme seçimlerini, sıcaklıkları ve hacimleri birbirine bağlar.", items: ["Dinlenmeleri, sıcaklıkları, süreleri ve infüzyon adımlarını içeren mayşe programı", "Seçilen modelle mayşe pH tahmini", "Mayşe hacmi ve sıcaklık adımları için dekoksiyon hesaplayıcısı", "Mayşe suyu, yıkama suyu ve hacim hesaplamaları", "BIAB'a özgü ekipman ve emilim yönetimi"] },
      { heading: "Maya hazırlığı", intro: "Maya gereksinimlerini ve fermantasyon beklentilerini görünür tutun.", items: ["Maya seçimi ve yaş bilgileri", "İki aşamalı starterlar dahil starter planlaması", "İşletme modunda hektolitre başına kuru maya aşılama rehberliği", "Fermantasyon profilleri ve hedef koşullar", "İşletme çekirdeğinde hasat ve nesil takibi; gelişmiş ev modunda maya yönetimi etkinleştirilebilir"] }
    ]
  },
  {
    slug: "brew-day",
    modules: ["core"],
    title: "Demleme günü ve üretim kayıtları",
    eyebrow: "DEMLEME GÜNÜ",
    summary: "Demleme günü çalışma alanı, reçete planını gerçekte olandan ayrı tutar; ıslak ekipmanın ve kâğıt kayıtların yanında da kullanılabilir kalır.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business core"],
    sections: [
      { heading: "Rehberli demleme günü zaman çizelgesi", intro: "Telefonda, tablette ya da webde net bir sıra izleyin.", items: ["Tam ekran adım adım iPhone akışı ve ayrıntılı iPad/web adım listesi", "İsteğe bağlı zamanlayıcılar, başlat/bitir denetimleri ve düzenlenebilir gerçek zamanlar", "Sonraki adım rehberliği, hedef değerler ve ölçüm girişi", "Desteklenen cihazlarda Apple Live Activity, Dynamic Island ve zamana duyarlı bildirimler", "Birden fazla parti aynı anda yürütülebilir"] },
      { heading: "Plan ve gerçekleşen", intro: "Parti, kaynak reçeteyi sessizce değiştirmeden gerçekte olanı kaydeder.", items: ["Gerçek miktarları, zamanı, sıcaklığı ve hacmi kaydedin", "Adım ekleyin, atlayın, yeniden sıralayın ya da düzeltin; malzeme ikamelerini ve eklemelerini kaydedin", "Sapmaları işaretleyin ve çevrimiçiyken gerçek OG, IBU ve ABV değerlerini yeniden hesaplayın", "Ölçülen değerleri hedeflerle karşılaştırın ve bağlama uygun yorum alın", "İsteğe bağlı olarak gerçekleşen sonucu yeni bir reçete taslağına dönüştürün"] },
      { heading: "Kâğıtla uyumlu akış", intro: "Dijital ve kâğıt akışlar birlikte çalışacak şekilde tasarlanmıştır.", items: ["Planlanan ve gerçekleşen alanları içeren, yazdırılabilir A4 veya US Letter üretim formları", "QR kodu basılı formu partisine bağlar", "Kâğıt üzerindeki ölçümleri aynı numaralı sırayla girin", "Üretim formlarını Apple belge tarama ile tarayın ve çıkarılan değerleri yan yana inceleyin", "İşletme çalışma alanları taranmış kayıtları zorunlu kılabilir ve saklama kuralları uygulayabilir", "Ev modunda üretim formu bölümlerini ve alanlarını özelleştirin", "Hatırlatıcılardan ve desteklenen sistem kısayollarından hızlı ölçüm girişi kullanın"] }
    ]
  },
  {
    slug: "fermentation-monitoring",
    modules: ["core"],
    title: "Fermantasyon ve parti takibi",
    eyebrow: "FERMANTASYON",
    summary: "Her partiyi demleme gününden fermantasyona, olgunlaştırmaya ve paketlemeye kadar; cihaz okumaları ya da elle yapılan ölçümlerle izleyin.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business core"],
    sections: [
      { heading: "Eksiksiz bir parti zaman çizelgesi", intro: "Her partinin kendi kaydı ve aşama geçmişi vardır.", items: ["Planlanan, üretilen, fermente olan, olgunlaşan, paketlemeye hazır, paketlenmiş ve kapatılmış aşamalarını izleyin", "Birden fazla partiyi ve fermantörü eş zamanlı yürütün", "Ölçümleri, notları, fotoğrafları ve olayları parti zaman çizelgesinde tutun", "Parti özetinde plan ve gerçek değerleri karşılaştırın", "Tadım notlarını kaydedin ve gelecekteki reçete çalışmalarına yön vermek için kullanın"] },
      { heading: "Ölçümler ve içgörüler", intro: "Okumaları tek bir görünümde bir araya getirin.", items: ["Cihazlardan ya da elle girişle alınan yoğunluk, sıcaklık ve pH okumaları", "Cihaz, elle ve kâğıt okumaları için ölçüm kaynağı etiketli grafikler", "Fermantasyon ilerleme vurguları", "Kuru şerbetçiotu ekleme, soğuk çöktürme ve paketleme zamanlaması önerileri", "Sensör bağlı olmadığında elle okuma hatırlatıcıları"] },
      { heading: "Kurallar ve uyarılar", intro: "Uyarılar sürece yanıt verir ve gereksiz gürültü üretmez.", items: ["Yoğunluğun birkaç gün sabit kalması ya da sıcaklığın hedef dışına çıkması gibi kurallar", "Üç öncelik düzeyi, sessiz saatler, gruplanmış uyarılar ve günlük özetler", "Onaylama, erteleme, yok sayma ve geçmiş içeren tek bir bildirim gelen kutusu", "İşletmede vardiya yönlendirmesi ve bir sonraki sorumlu kişiye yükseltme", "Elle okuma hatırlatıcısından doğrudan hızlı SG ve sıcaklık girişi", "Fermantasyon durumunu bir bakışta göstermek için ev ve tank odaklı widget'lar"] },
      { heading: "Parti karşılaştırması ve olgunlaştırma", intro: "Tamamlanan partileri, zaman içindeki tekrarlanabilirliği anlamak için kullanın.", items: ["Aynı reçeteden yapılan partileri OG, FG, verim ve tadım puanına göre karşılaştırın", "Ev modunda şişe ve keg olgunlaştırmasının hedef karbonasyona doğru ilerlemesini izleyin", "Ev modunda tadım geri bildirimini bir reçete taslağına dönüştürün"] }
    ]
  },
  {
    slug: "equipment-calibration",
    modules: ["devices"],
    title: "Ekipman ve kalibrasyon",
    eyebrow: "EKİPMAN",
    summary: "Ekipman profilleri, bir ev kurulumunun ya da profesyonel demhanenin gerçek davranışını yansıtır ve ölçülen parti sonuçlarıyla gelişir.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business core"],
    sections: [
      { heading: "Ekipman profilleri", intro: "Her hesaplamayı şekillendiren ekipmanı tanımlayın.", items: ["İlk profil için rehberli ekipman kurulumu", "Mayşe tankı, kazan, fermantör ve paketleme kabı ayrıntıları", "Hacim kayıpları, tahıl emilimi, buharlaşma, ölü hacim ve soğutma büzülmesi", "İşletme çalışma alanları için birden fazla demhane ve kap profili", "Ekipmana özgü reçete ölçekleme ve verim varsayımları"] },
      { heading: "Kap hazırlığı", intro: "Profesyonel kap ayrıntıları ekipman planlamasını üretime bağlar.", items: ["Tank kapasitesi, basınç sınırı ve mevcut doluluk", "Temizlik durumu ve son temizlik kaydı", "Üretim takvimindeki kap kullanılabilirliği", "Ekipman kalibrasyon kayıtları"] }
    ]
  },
  {
    slug: "planning-scheduling",
    modules: ["production"],
    title: "Planlama ve çizelgeleme",
    eyebrow: "İŞİ PLANLAYIN",
    summary: "Hedef paketleme tarihinden geriye doğru planlayın, her adım için gereken süreyi anlayın ve partilerin ekipman kullanılabilirliğine nasıl sığdığını görün.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business core", "Business second release"],
    sections: [
      { heading: "Takvim ve geriye dönük planlama", intro: "Biranın hazır olmasını istediğiniz tarihten geriye çalışın.", items: ["Demleme günü, fermantasyon adımları, kuru şerbetçiotu ekleme, soğutma ve paketleme tarihlerini planlayın", "Fermantasyon profillerinden hatırlatıcılar zamanlayın", "Isıtma ve soğutma süresini güç, hacim ve soğutma ekipmanından tahmin edin", "Zamanlama tahminlerini iyileştirmek için geçmiş ekipman performansını kullanın", "Kapasite ve doluluk çakışması uyarılarıyla işletme tank takvimi"] },
      { heading: "Paralel partiler", intro: "Eş zamanlı işleri görünür tutun.", items: ["Partileri aşamaya ve yaklaşan göreve göre görün", "Tüm etkin partilerdeki görevleri içeren tek bir “Bugün” listesi", "Haftalık tank takvimi ve günlük tank turu çıktısı", "Bir partiyi atamadan önce tank çakışmalarını belirleyin", "İşletmede tank boşaltma önerileri ve kapasite optimizasyonu"] },
      { heading: "Esnek görünümler", intro: "İşletme üretim panosu ekibe göre uyum sağlar.", items: ["Düzenlenebilir pano kartları ve sıralaması", "Role özgü varsayılan görünümler", "Yalnızca etkin fermantasyonlar gibi kayıtlı görünümler", "Operatör odaklı Bugün görünümü ve duvara monte ekran modu"] }
    ]
  },
  {
    slug: "inventory-costing",
    modules: ["stock","cost-reports"],
    title: "Envanter, malzemeler ve maliyet",
    eyebrow: "MALZEME VE MALİYET",
    summary: "Malzemeleri basit bir ev stok listesinden profesyonel lot düzeyinde izlenebilirliğe kadar yönetin; reçete ve parti maliyetleri gerçek malzeme değerlerine dayanır.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business core", "Business second release", "Under consideration"],
    sections: [
      { heading: "Malzeme kütüphanesi", intro: "Ortak bir malzeme kataloğu reçete tasarımını ve satın almayı destekler.", items: ["Malt, fermente edilebilir malzemeler, şerbetçiotu, maya, su ve çeşitli malzemeler", "Ayrıntılı malzeme özellikleri ve özel malzemeler", "Malzeme ikameleri ve stoğa duyarlı reçete planlaması", "Malzemeye özgü birimler ve bölgesel gösterim", "İşletme modunda özel malt analizi ve analiz sertifikası (COA)"] },
      { heading: "Stok ve mal kabul", intro: "Stok yönetimi çalışma alanı moduyla birlikte ölçeklenir.", items: ["Ev stok miktarları, asgari seviyeler ve alışveriş listesi", "İşletmede stok lotları, tedarikçi bilgileri, rezervasyonlar ve FIFO değerlemesi", "Mal kabul, barkod desteği ve stok sayımları", "Girişler, kullanım, transferler ve düzeltmeler için hareket defteri", "Üretim için lot kullanılabilirliği"] },
      { heading: "Reçete ve parti maliyetleri", intro: "Malzeme maliyetini önem taşıyan düzeyde anlayın.", items: ["Ev biracıları için isteğe bağlı reçete maliyeti", "FIFO lot fiyatlarıyla işletmede parti ve litre başına maliyet", "Ambalaj boyutuna göre şişe ve servis maliyeti", "FIFO lot fiyatlarına ve kaydedilen miktarlara dayanan maliyet tahminleri"] },
    ]
  },
  {
    slug: "production-management",
    modules: ["production"],
    title: "Bira üretim yönetimi",
    eyebrow: "İŞLETME MODU",
    summary: "Küçük bira üreticileri için bir üretim çalışma alanı; reçetelere, partilere, tanklara, kişilere ve kayıtlara odaklanır, genel amaçlı bir ERP değildir.",
    audiences: ["Breweries"],
    stages: ["Business core", "Business second release"],
    sections: [
      { heading: "Üretim panosu ve operatör görünümü", intro: "Her role bugün nelerin dikkat gerektirdiğine dair yararlı bir görünüm sunun.", items: ["Parti aşamasına ve tanka göre düzenlenmiş üretim panosu", "Operatörler için Bugün görev listesi", "Role dayalı pano ve kayıtlı görünümler", "Üretim sahasındaki iPad'ler için operatör ekranı", "Günlük tank turları ve yazdırılabilir haftalık tank takvimi"] },
      { heading: "Tank ve parti hareketleri", intro: "Biranın bira üretim tesisi içindeki fiziksel yolculuğunu modelleyin.", items: ["Doluluğu, durumu, basınç sınırını ve temizlik durumunu içeren tank ayrıntısı", "Hacim ve kayıp kayıtlarıyla kaplar arası transfer", "Bir partiyi birden fazla tanka bölme", "Hesaplanan harman özellikleriyle partileri birleştirme", "Kapasite çakışması kontrolleri ve tank çizelgeleme"] },
      { heading: "Üretim kayıtları", intro: "Planlanan süreci ve gerçekleşen işi birbirine bağlı tutun.", items: ["Operatör kimliği, adım tamamlama ve gerçek ölçümler", "Sapma kayıtları ve düzeltici kararlar", "Malzemelerden paketlemeye partinin izlenebilirliği; lottan partiye geri çağırma raporlarıyla", "Birden fazla operatör ve eş zamanlı etkin partiler", "Kapatılmış üretim kayıtlarındaki değişiklikler için denetim izi", "Geçerli temizlik kaydı olmayan bir tank atandığında uyarılar", "Açık uyarıların ve üretim notlarının vardiya devri"] }
    ],
    note: "GRISTORA bir ERP değildir. Satış, sipariş, dağıtım, müşterideki fıçının takibi ve depozitosu, CRM ve muhasebe ürün kapsamının dışındadır."
  },
  {
    slug: "quality-laboratory",
    modules: ["quality"],
    title: "Kalite, laboratuvar ve serbest bırakma",
    eyebrow: "KALİTE YÖNETİMİ",
    summary: "Profesyonel kalite akışları; ürün spesifikasyonları, laboratuvar sonuçları, tadım ve paketlemeden önce kayda geçirilen bir serbest bırakma kararı etrafında kurulur.",
    audiences: ["Breweries"],
    stages: ["Business core", "Business second release", "Under consideration"],
    sections: [
      { heading: "Ürün spesifikasyonları", intro: "Her bira için ölçülebilir hedef aralıkları tanımlayın.", items: ["ABV, OG, FG, IBU, renk ve pH için reçeteye bağlı spesifikasyonlar", "Çözünmüş oksijen, bulanıklık, diasetil ve mikrobiyoloji parametreleri", "Aralık dışı uyarılar ve sapma kayıtları", "Partiye ve reçete sürümüne bağlı kalite incelemesi"] },
      { heading: "Laboratuvar ve duyusal kayıtlar", intro: "Analitik ve tadım sonuçlarını partiyle birlikte tutun.", items: ["pH, çözünmüş oksijen, CO₂, ABV ve mikrobiyoloji için laboratuvar sonuçları", "Tadım notları, duyusal puan ve tanımlayıcı nitelikler", "Geçmiş sonuçların sessizce düzenlenmesi yerine düzeltme kayıtları", "Duyusal panel ve üçgen test akışları"] },
      { heading: "Serbest bırakma ve süreç kontrolü", intro: "Paketleme kararını açık ve izlenebilir hale getirin.", items: ["Gerekçeleriyle kalite bekletme, ret ve serbest bırakma kararları", "İşletme akışlarında paketlemeden önce serbest bırakma onayı", "HACCP kontrol noktaları ve çift imza"] },
    ]
  },
  {
    slug: "packaging-compliance",
    modules: ["production","quality","tax-compliance"],
    title: "Paketleme, CIP ve uyumluluk",
    eyebrow: "PAKETLEME VE KAYITLAR",
    summary: "Paketleme kayıtları, kap temizliği ve yargı bölgesine duyarlı raporlama profesyonel üretimi destekler; sistemin neleri yapmadığı da açıkça belirtilir.",
    audiences: ["Breweries"],
    stages: ["Business core", "Business second release"],
    sections: [
      { heading: "Paketleme çalışmaları ve kaplar", intro: "Neyin, hangi formatta ve ne kadar paketlendiğini kaydedin.", items: ["Yapılandırılabilir boyutlarla şişe, kutu, keg ve UK cask kap türleri", "Paketleme miktarları, litreler, kayıplar, paketleme verimi ve lot kodları", "Tek bir parti içinde birden fazla kap türü", "Şişe priming, keg basınçlı karbonasyon, BBT'den kutulama ve cask olgunlaştırma rehberliği", "Paketleme raporlaması için ambalaj malzemesi ve boş ağırlık"] },
      { heading: "Temizlik ve mahzen süreci", intro: "Temizlik kayıtlarını kap hazırlığına bağlayın.", items: ["CIP kimyasalı, konsantrasyon, sıcaklık, süre ve doğrulama", "Tank temizlik durumu ve son temizlenme kaydı", "Geçerli temizlik kaydı olmayan bir tank atandığında uyarılar", "Spunding ve zorlamalı karbonasyon planlaması", "UK pazarı için cask olgunlaştırma ve durultucu (finings) hesaplamaları"] },
      { heading: "Etiketler ve mevzuat raporları", intro: "Muhasebe veya satış sistemlerinin yerini almaya çalışmadan üretim tarafı raporlamasını destekleyin.", items: ["Ülkeye özgü vergi hesaplama yöntemleri ve raporlama şablonları", "ABD TTB, Birleşik Krallık HMRC, Avustralya ATO, Almanya Zoll ve Türkiye raporlama kapsamı", "Yargı bölgesine göre ABV veya Plato tabanlı vergi hesaplamaları", "Gereklilikler doğrulandığında ABV, lot kodu, alerjenler ve besin değerleri gibi etiket bilgileri", "Depozito iadesi ve ambalaj/EPR raporları"] }
    ],
    note: "Satış ve sevkiyat miktarlarının elle girilmesi ya da dosyayla içe aktarılması gerekebilir; çünkü satış ve dağıtım kapsam dışındadır."
  },
  {
    slug: "team-notifications",
    modules: ["team"],
    title: "Ekipler, roller ve bildirimler",
    eyebrow: "İNSANLAR VE İŞ AKIŞLARI",
    summary: "Tek bir çalışma alanı hem tek başına çalışan bir biracıyı hem de çok rollü bir üretim ekibini destekler; bildirimler doğru kişiye yönlendirilir ve eylemler kaydedilir.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business core"],
    sections: [
      { heading: "Ev ve işletme çalışma alanları", intro: "Çalışma alanı, temeldeki reçete ve parti geçmişini bölmeden Ev ya da İşletme modunu seçer.", items: ["İlk kurulumda bir mod seçin ve daha sonra çalışma alanı ayarlarından değiştirin", "Ev modu gelişmiş işletme alanlarını görünümden uzak tutar", "Ev modunda bir reçete veya parti üzerinde iş birliği için isteğe bağlı davetler", "Mod değiştirmek reçeteleri ve geçmişi korur; gizlenen işletme kayıtları silinmez"] },
      { heading: "Ekip, roller ve paylaşılan cihazlar", intro: "İşletme çalışma alanları role dayalı üretim işini destekler.", items: ["Süreli davetlerle kullanıcıları davet edin ve bir ya da daha fazla rol atayın", "Sahip, usta demci, operatör, kalite, depo ve salt okunur denetçi rolleri", "Özel roller, kopyalanan roller, izin grupları ve isteğe bağlı kap/alan kapsamı", "Kişi bazında izin özeti ve role özgü açılış ekranları", "Kişisel PIN, hızlı kullanıcı geçişi ve otomatik kilitle paylaşılan üretim iPad'i"] },
      { heading: "Bildirimler ve onaylar", intro: "Tek bir bildirim kaydı cihazlar arasında tutarlı kalır.", items: ["Uygulama içi gelen kutusu, anlık bildirimler ve isteğe bağlı günlük e-posta özeti", "Üç öncelik düzeyi, sessiz saatler, gruplanmış uyarılar ve bildirim geçmişi", "Bildirimleri bir olay kaydıyla onaylayın, erteleyin, yok sayın ya da çözün", "İşletmede vardiya yönlendirmesi, yükseltme ve vardiya devri", "Reçete onayı, kalite serbest bırakma ve diğer role dayalı onay kuyrukları"] }
    ]
  },
  {
    slug: "connected-devices",
    modules: ["devices","connections"],
    title: "Bağlı cihazlar ve entegrasyonlar",
    eyebrow: "BAĞLI CİHAZLAR",
    summary: "Salt okunur fermantasyon cihazı verileri, profesyonel sensör ve endüstriyel ağ geçidi desteğiyle birlikte sunulur.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business second release", "Later", "Under consideration"],
    sections: [
      { heading: "Evde bira yapım cihazları", intro: "Yaygın fermantasyon monitörlerini özel adaptörlerle bağlayın.", items: ["Tilt hidrometreler, iSpindel ve RAPT cihazları", "TiltBridge, Pitch ve uyumlu köprüler için webhook veri uç noktası", "Yenilenebilen, cihaza özgü anahtarlar", "Cihaz-parti eşleştirmesi ve kalibrasyon kayıtları", "Bağlantı ve okuma durumunu gösterin"] },
      { heading: "Profesyonel sensörler", intro: "İşletme modu tank düzeyinde izleme ekler.", items: ["Yoğunluk, pH, çözünmüş oksijen ve basınç okumaları", "Tank ataması ve temel bağlantı durumu", "Profesyonel sensörler için API ya da CSV ile veri alımı", "Modbus TCP, OPC UA ve MQTT için endüstriyel ağ geçidi okuma desteği", "Ekipmana kontrol komutu göndermek kapsam dışındadır"] },
      { heading: "Veri kullanımı ve güvenilirlik", intro: "Sensör bağlantısı kesildiğinde de cihaz verileri yararlı kalmalıdır.", items: ["Sensör okumalarını cihazda ya da ağ geçidinde tamponlayın ve bağlantı kurulunca iletin", "Ölçümlerin yanında kaynak etiketlerini ve kalibrasyon bağlamını gösterin", "Cihaz adaptörlerini tek bir üreticiden bağımsız tutun", "Okumaları fermantasyon eğilimleri ve yapılandırılmış eşik uyarıları için kullanın", "İşletme modu için açık API ve profesyonel üretim verisi webhook'ları; ev biracılığı cihaz webhook'ları ayrıdır", "Endüstriyel ağ geçidi desteği salt okunurdur; ekipman komutu ve kontrolü kapsam dışındadır"] },
    ]
  },
  {
    slug: "imports-exports-tools",
    modules: ["core","connections"],
    title: "İçe aktarma, dışa aktarma ve bira yapım araçları",
    eyebrow: "TAŞINABİLİRLİK VE ARAÇLAR",
    summary: "Mevcut reçete çalışmalarınızı GRISTORA'ya taşıyın, gerektiğinde verilerinizi dışarı alın ve reçete ile parti akışlarının yanında odaklı bira yapım araçlarını kullanın.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Homebrew second release"],
    sections: [
      { heading: "Reçete taşınabilirliği", intro: "İçe aktarma yolları yerleşik bira dosya biçimlerini kapsar.", items: ["BeerXML ve BeerJSON içe/dışa aktarma", "BeerSmith .bsmx içe aktarma", "Brewfather ve Brewer's Friend reçete dışa aktarımları", "İçe aktarma sırasında malzeme, ekipman, stil ve birim eşleştirmesi", "Kaydetmeden önce kaynak ile GRISTORA hesaplama farklarını gösterin"] },
      { heading: "Odaklı bira yapım araçları", intro: "Odaklı yardımcı araçlar ilgili reçete ya da parti bağlamında yer alır.", items: ["Refraktometre düzeltmesi ve yoğunluk araçları", "Şişeler, kegler ve caskler için karbonasyon ve priming hesaplayıcıları", "Seyreltme, hacim ve birim dönüşümü", "Kısmi kaynatma, tamamlama suyu ve LME/DME/bal dönüşümleri için ekstrakt bira yapım araçları", "Dekoksiyon ve parti-gyle araçları; parti-gyle Ev modunda kullanılabilir", "Ev modunda no-chill IBU ayarı ve şişe/keg karbonasyon takibi", "Ev modunda desteklenen bira kontrol cihazlarına isteğe bağlı reçete aktarımı"] },
      { heading: "Raporlar ve özel kayıtlar", intro: "Yararlı kayıtları taşınabilir ve yazdırılabilir hale getirin.", items: ["Üretim formları, tank turları, fermantasyon ve paketleme formları", "Ev modunda özel üretim formu alanları ve yerleşimleri", "Reçete, parti, ekipman, su profili, malzeme, cihaz ve destek verilerinin dışa aktarımı", "CSV ve JSON dışa aktarımlarının yanı sıra taranmış belgeler ve fotoğraflar, indirilebilir arşivlerde", "Ev modunda yarışma formu çıktısı ve özel reçete/parti alanları", "Ev modunda laboratuvar su raporlarını içe aktarın"] }
    ]
  },
  {
    slug: "platform-security-data",
    modules: ["connections"],
    title: "Platform, güvenlik ve veri sahipliği",
    eyebrow: "PLATFORM",
    summary: "Ürün, Apple cihazları ve masaüstü web arasında tek bir bağlantılı sistemdir; ortak bir sunucu tarafı motoru ve açık veri sahipliği denetimleri vardır.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business core"],
    sections: [
      { heading: "GRISTORA nerede çalışır", intro: "Her istemcinin kendi rolü vardır.", items: ["iPad: tam özellikli yerel SwiftUI uygulaması", "iPhone: demleme günü, ölçüm, fermantasyon, stok ve alışveriş akışları; reçeteler salt okunur görünür, reçete tasarımı ve planlama iPad ve web'de", "Web: Mac ve PC tarayıcıları için tam özellikli React ve TypeScript deneyimi", "Apple Watch desteklenmez", "Android ve ayrı Mac/Windows masaüstü uygulamaları kapsam dışındadır"] },
      { heading: "Tek motor, tek doğruluk kaynağı", intro: "Aynı hesaplama ve iş kuralları her istemciye hizmet eder.", items: ["Rust hesap motoru sunucu tarafında çalışır; hesaplama kodu istemcilere gönderilmez", "Web ve Apple uygulamaları arasında ortak API ve veri modeli", "Önce çevrimiçi çalışma; çevrimdışı zamanlayıcılar ve ölçüm girişi için sınırlı, indirilen bir demleme günü paketi", "Genel çevrimdışı senkronizasyon yoktur; kuyruğa alınan demleme günü ölçümleri bağlantı geri geldiğinde yüklenir", "Sürümlü hesaplama sonuçları ve değiştirilemez reçete/parti referansları"] },
      { heading: "Güvenlik ve veri sahipliği", intro: "Güvenlik ve taşınabilirlik platformun parçasıdır.", items: ["Apple veya Google ile giriş; yedek olarak tek kullanımlık e-posta bağlantıları; şifre tabanlı kayıt yoktur", "İşletme yöneticileri için passkey'ler, oturum yönetimi ve uzaktan oturum kapatma", "Çalışma alanı ayrımı, sunucu tarafı izinler ve yalnızca eklemeli denetim kayıtları", "Şifreli depolama, korumalı dosya erişimi ve müşteri onayıyla süreli destek erişimi", "Reçetelerinizi, partilerinizi, ölçümlerinizi, stokunuzu, ekipmanınızı ve dosyalarınızı dışa aktarın; hesap silme talep edin"] },
      { heading: "Destek ve hizmet", intro: "Müşteri desteği ürün deneyiminin parçası olmaya devam eder.", items: ["Aranabilir yardım ve destek talebi gelen kutusu", "Müşterinin görebildiği talep geçmişi ve yanıtlar", "Gerekçe ve denetim geçmişiyle isteğe bağlı, süreli destek erişimi", "Müşteri uygulamasından ayrılmış dahili destek çalışma alanı", "Sistem durumu ve destek tanılamaları"] },
      { heading: "Çalışma alanı kurulumu, tercihler ve abonelik", intro: "İlk çalıştırma ve hesap deneyimi, ayrı bir kayıt formu yerine bir çalışma alanı etrafında şekillenir.", items: ["Rehberli ilk açılış: Ev ya da İşletme modunu, veri bölgesini ve ekipman profilini seçin", "Reçeteler, partiler, malzemeler ve stok genelinde genel arama", "Bölgesel ön ayarlar, kategoriye özgü birimler, yerel sayı/tarih biçimleri ve İngilizce/Türkçe dil desteği", "Tema, görünüm, bildirim, ekipman, hesaplama ve stil rehberi ayarları", "Abonelik ve hak yönetimi", "Hesap bağlama, oturum yönetimi, uzaktan oturum kapatma, hesap dışa aktarma ve silme"] },
      { heading: "Apple ve web deneyimi", intro: "Platformun desteklediği yerlerde sistem düzeyinde kolaylıklar.", items: ["iOS Live Activities, Dynamic Island, widget'lar, zamana duyarlı bildirimler ve belge tarama", "Apple deneyiminin parçası olarak Siri ve Kısayollar entegrasyonu", "Apple Watch desteklenmez; Android ve ayrı masaüstü uygulamaları kapsam dışındadır", "Face ID veya PIN ile uygulama kilidi, paylaşılan cihaz PIN'i ve yönetici passkey'leri", "Şifreli yedeklemeler, veri taşınabilirliği ve müşterinin görebildiği destek erişim geçmişi"] }
    ]
  },
  {
    slug: "analytics-reports",
    modules: ["cost-reports"],
    title: "Raporlar, içgörüler ve sürdürülebilirlik",
    eyebrow: "RAPORLAMA VE İÇGÖRÜLER",
    summary: "Operasyonel raporlama; partileri, reçeteleri ve bira üretimini kapsar.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business core", "Business second release", "Under consideration"],
    sections: [
      { heading: "Parti ve reçeteden öğrenme", intro: "Tekrarlanabilirliği anlamak için gerçek sonuçları kullanın.", items: ["Plan ve gerçekleşen parti özeti ile tadım notları", "Aynı reçeteden yapılan partileri karşılaştırın", "Ev modunda yoğunluk, verim ve tadım puanı eğilimleri"] },
      { heading: "Bira üretimi raporlaması", intro: "İşletme raporlaması üretime ve malzeme performansına odaklanır.", items: ["Demhane, dönüşüm ve süzme verimi", "Demleme gününden paketlemeye hacim kaybı", "FIFO malzeme maliyeti ve litre başına maliyet", "Tank kullanımı, parti çıktısı ve üretim panosu metrikleri", "Malzeme lotundan paketleme çalışmasına lot izlenebilirliği ve geri çağırma raporu"] }
    ]
  }
];
