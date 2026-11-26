export const screens = {
  "ed": {
    "tr": [
      "Tarif editörü",
      "Üretici, demlemeye başlamadan önce tarif hedeflerini gözden geçirir."
    ],
    "en": [
      "Recipe editor",
      "Before brewing, the brewer reviews the recipe targets."
    ],
    "width": 1209,
    "height": 842
  },
  "bd": {
    "tr": [
      "Demleme günü",
      "Operatör aktif adımı ve zamanlayıcıyı iPad üzerinden takip eder."
    ],
    "en": [
      "Brew day",
      "On the brew floor, the operator follows the current step and timer on iPad."
    ],
    "width": 1209,
    "height": 842
  },
  "fe": {
    "tr": [
      "Fermantasyon özeti",
      "Bir sonraki vardiya tankın ölçümlerini aynı kayıtta bulur."
    ],
    "en": [
      "Fermentation overview",
      "The next shift can look up the tank readings in one record."
    ],
    "width": 1209,
    "height": 842
  },
  "bs": {
    "tr": [
      "Parti özeti",
      "Üretici partiyi kapatırken hedeflerle kaydedilen sonuçları karşılaştırır."
    ],
    "en": [
      "Batch summary",
      "At closeout, the brewer compares the targets with the recorded results."
    ],
    "width": 1209,
    "height": 929
  },
  "pr": {
    "tr": [
      "Üretim panosu",
      "Vardiyanın ilk sorusu hangi tankın dikkat gerektirdiğidir."
    ],
    "en": [
      "Production board",
      "At handover, the team first checks which tank needs attention."
    ],
    "width": 1209,
    "height": 875
  },
  "tk": {
    "tr": [
      "Tank detayı",
      "Tank kartı, ekibi ilgili parti kaydına götürür."
    ],
    "en": [
      "Tank detail",
      "A tank card puts the associated batch in context."
    ],
    "width": 1716,
    "height": 924
  },
  "qa": {
    "tr": [
      "Kalite kaydı",
      "Serbest bırakma kararından önce ölçüm ve spesifikasyon birlikte incelenir."
    ],
    "en": [
      "Quality record",
      "Before release, the team reviews the measurement alongside its specification."
    ],
    "width": 1716,
    "height": 1017
  },
  "inv": {
    "tr": [
      "Lot bazında stok",
      "Üretici eldeki malzemeyi lot kaydından kontrol eder."
    ],
    "en": [
      "Inventory by lot",
      "The brewer checks available ingredients by lot."
    ],
    "width": 1209,
    "height": 902
  },
  "lt": {
    "tr": [
      "Lot izlenebilirliği",
      "Bir malzeme lotundan onu kullanan partilere gidilir."
    ],
    "en": [
      "Lot traceability",
      "Follow an ingredient lot to the batches that used it."
    ],
    "width": 1716,
    "height": 615
  },
  "wiz": {
    "tr": [
      "Tarif sihirbazı",
      "Ev biracısı bir sonraki demlemeye tarifin başlangıç seçimleriyle hazırlanır."
    ],
    "en": [
      "Recipe wizard",
      "The homebrewer prepares for the next brew by choosing a recipe starting point."
    ],
    "width": 1209,
    "height": 842
  },
  "cmp": {
    "tr": [
      "Tarif sürümleri",
      "İki sürümü yan yana görmek bir sonraki demlemede neyin değişeceğini açıklar."
    ],
    "en": [
      "Recipe versions",
      "Two versions side by side show what will change in the next brew."
    ],
    "width": 1716,
    "height": 677
  },
  "cal": {
    "tr": [
      "Üretim takvimi",
      "Demleme günü öncesindeki işleri takvimde sıralayın."
    ],
    "en": [
      "Brewing calendar",
      "Read the schedule leading up to brew day."
    ],
    "width": 1209,
    "height": 764
  },
  "tools": {
    "tr": [
      "Ölçüm aracı",
      "Sonucu yorumlamadan önce ölçüm girdisini ve birimini kontrol edin."
    ],
    "en": [
      "Measurement tool",
      "Check the input and its unit before interpreting the result."
    ],
    "width": 1209,
    "height": 791
  },
  "hw": {
    "tr": [
      "Değer açıklaması",
      "Bir değerin nasıl elde edildiğini tarif üzerinde inceleyin."
    ],
    "en": [
      "Value explanation",
      "Inspect how a value is derived in the context of the recipe."
    ],
    "width": 1209,
    "height": 842
  },
  "cip": {
    "tr": [
      "Temizlik ve dolum kaydı",
      "Parti kapanırken temizlik ve dolum kayıtlarını gözden geçirin."
    ],
    "en": [
      "Cleaning and packaging record",
      "Review cleaning and packaging records as the batch closes."
    ],
    "width": 1209,
    "height": 792
  },
  "cl": {
    "tr": [
      "Ekipman kalibrasyonu",
      "Ölçümü yorumlarken kullanılan ekipman profilini kontrol edin."
    ],
    "en": [
      "Equipment calibration",
      "Check the equipment profile behind a measurement."
    ],
    "width": 1209,
    "height": 1137
  }
} as const;

export const placements: Record<string, readonly (keyof typeof screens)[]> = {
  "home": [
    "ed",
    "bd",
    "bs"
  ],
  "how-it-works": [
    "cal",
    "bd",
    "fe",
    "bs"
  ],
  "breweries": [
    "pr",
    "tk",
    "qa"
  ],
  "homebrewers": [
    "wiz",
    "bd",
    "cmp"
  ],
  "modules": [
    "inv",
    "qa",
    "pr"
  ],
  "technical-principles": [
    "tools",
    "hw"
  ],
  "compare": [
    "wiz",
    "pr"
  ],
  "features/recipe-design": [
    "ed",
    "cmp"
  ],
  "features/brew-day": [
    "bd"
  ],
  "features/fermentation-monitoring": [
    "fe"
  ],
  "features/production-management": [
    "pr",
    "bs",
    "tk"
  ],
  "features/quality-laboratory": [
    "qa"
  ],
  "features/inventory-costing": [
    "inv",
    "lt"
  ],
  "features/planning-scheduling": [
    "cal"
  ],
  "features/brewing-calculations": [
    "tools"
  ],
  "features/packaging-compliance": [
    "cip"
  ],
  "features/equipment-calibration": [
    "cl"
  ]
};
