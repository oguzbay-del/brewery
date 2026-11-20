// Representative AI-generated photos (owner decision LCH-07, 20 Nov 2026).
// Raw files: ../fotograflar-ham/. Build outputs: scripts/optimize-photos.py.
export type PhotoKey = keyof typeof photos;

export const photos = {
  "home-hero": { en: "A row of stainless-steel conical fermenters in a brick-walled brewery cellar, with a brewer checking a tank.", tr: "Tuğla duvarlı bir bira mahzeninde sıra hâlinde paslanmaz çelik konik fermentörler ve tankı kontrol eden bir bira üreticisi." },
  "breweries-hero": { en: "A brewer seen from behind at a three-vessel stainless-steel brewhouse.", tr: "Üç kazanlı paslanmaz çelik bir brewhouse önünde arkadan görülen bir bira üreticisi." },
  "homebrewers-hero": { en: "A home kitchen brewing setup with a kettle on an induction plate, a glass fermenter, malt and hops.", tr: "Ev mutfağında indüksiyon ocak üstünde kazan, cam fermentör, malt ve şerbetçiotuyla bira yapım düzeni." },
  "features-hero": { en: "Malted barley, hop cones, a yeast vial, a refractometer and a blank notebook on a dark slate surface.", tr: "Koyu arduvaz zemin üstünde malt, şerbetçiotu kozalakları, maya şişesi, refraktometre ve boş bir defter." },
  "how-it-works-hero": { en: "A brewer pours a beer sample from a fermenter sample valve into a glass.", tr: "Bir bira üreticisi fermentörün numune vanasından bardağa numune alıyor." },
  "compare-hero": { en: "A golden pale ale and a deep amber ale side by side on a wooden bar.", tr: "Ahşap bar üstünde yan yana altın renkli bir pale ale ve koyu amber bir ale." },
  "pricing-hero": { en: "A small taproom brewery with fermenters behind glass and wooden bar stools.", tr: "Camın arkasında fermentörler ve ahşap bar tabureleriyle küçük bir taproom bira fabrikası." },
  "faq-hero": { en: "A brewer at a wooden desk with a coffee cup and a beer glass, fermenters visible through the window.", tr: "Kahve fincanı ve bira bardağıyla ahşap masada oturan bir bira üreticisi; pencereden fermentörler görünüyor." },
  "get-started-hero": { en: "A gloved hand adds yeast to fresh wort in an open stainless-steel vessel.", tr: "Eldivenli bir el, açık paslanmaz çelik kapta taze şıraya maya ekliyor." },
  "technical-principles-hero": { en: "A brewing lab bench with a digital scale, glass beakers and a pH probe.", tr: "Dijital terazi, cam beherler ve pH probuyla bir bira laboratuvarı tezgâhı." },
  "404": { en: "Looking into an empty, clean stainless-steel fermenter through its top hatch.", tr: "Üst kapağından bakılan boş ve temiz bir paslanmaz çelik fermentör." },
  "feature-recipe-design": { en: "Bowls of different malts and hops in a row beside a blank recipe notebook.", tr: "Boş bir tarif defterinin yanında sıra hâlinde farklı malt ve şerbetçiotu kâseleri." },
  "feature-brewing-calculations": { en: "A brewer reads a refractometer held up to the light.", tr: "Bir bira üreticisi ışığa tuttuğu refraktometreyi okuyor." },
  "feature-water-mash-yeast": { en: "A brewer stirs the mash with a paddle while a thermometer probe sits in the mash.", tr: "Bir bira üreticisi termometre probu takılı mayşeyi kürekle karıştırıyor." },
  "feature-brew-day": { en: "Hop pellets poured from a scoop into a steaming kettle.", tr: "Buharı tüten kazana kepçeyle şerbetçiotu peleti ekleniyor." },
  "feature-equipment-calibration": { en: "Gloved hands check a pressure gauge on top of a stainless-steel tank, with a wrench nearby.", tr: "Eldivenli eller paslanmaz çelik tankın üstündeki basınç göstergesini kontrol ediyor; yanında bir anahtar var." },
  "feature-production-management": { en: "A brewer walks along an aisle between two rows of tall fermenters.", tr: "Bir bira üreticisi iki sıra uzun fermentörün arasındaki koridorda yürüyor." },
  "feature-inventory-costing": { en: "Stacked malt sacks on pallets and boxed supplies on shelves in a brewery storeroom.", tr: "Bira fabrikası deposunda paletlere istiflenmiş malt çuvalları ve raflarda kutulu malzemeler." },
  "feature-fermentation-monitoring": { en: "Amber beer fermenting in a glass jug closed with a bung and an airlock.", tr: "Tıpa ve hava kilidiyle kapatılmış cam damacanada mayalanan amber bira." },
  "feature-planning-scheduling": { en: "A brewer walks past a row of conical fermenters in a brick-walled cellar.", tr: "Bir bira üreticisi tuğla duvarlı mahzende sıra hâlindeki konik fermentörlerin yanından geçiyor." },
  "feature-quality-laboratory": { en: "A lab technician at a microscope with petri dishes and a rack of beer samples.", tr: "Mikroskop başında petri kapları ve bira numunesi rafıyla çalışan bir laboratuvar teknisyeni." },
  "feature-packaging-compliance": { en: "Unlabelled aluminium cans on a small canning line.", tr: "Küçük bir kutulama hattında etiketsiz alüminyum kutular." },
  "feature-team-notifications": { en: "Two brewers in aprons check a valve on a fermenter together.", tr: "Önlüklü iki bira üreticisi fermentördeki bir vanayı birlikte kontrol ediyor." },
  "feature-connected-devices": { en: "A gloved hand fits a small sensor to the cone of a stainless-steel fermenter.", tr: "Eldivenli bir el, paslanmaz çelik fermentörün konik kısmına küçük bir sensör takıyor." },
  "feature-imports-exports-tools": { en: "Old handwritten brew logbooks and recipe cards next to a laptop.", tr: "Dizüstü bilgisayarın yanında eski el yazısı bira defterleri ve tarif kartları." },
  "feature-platform-security-data": { en: "A key in the lock of a stainless-steel cabinet in a brewery.", tr: "Bir bira fabrikasındaki paslanmaz çelik dolabın kilidinde bir anahtar." },
  "feature-analytics-reports": { en: "Five glasses of amber beer in a row, from lighter to darker, for batch comparison.", tr: "Parti karşılaştırması için açıktan koyuya sıralanmış beş bardak amber bira." },
} as const;

// Stand-ins for a feature without a dedicated photo. Empty: every feature has its photo.
const featureFallback: Record<string, PhotoKey> = {};

/** Photo for a feature slug: the dedicated photo, else a temporary stand-in. */
export function featurePhoto(slug: string): PhotoKey | undefined {
  const key = `feature-${slug}`;
  return key in photos ? (key as PhotoKey) : featureFallback[slug];
}
