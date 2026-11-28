export const galleryText = {
  title: 'GRISTORA UI Kit',
  theme: 'Tema / Theme',
  themes: [
    { value: 'light', label: 'Açık / Light' },
    { value: 'dark', label: 'Koyu / Dark' },
    { value: 'contrast', label: 'Yüksek karşıtlık / High contrast' },
    { value: 'contrast-dark', label: 'Koyu yüksek karşıtlık / Dark high contrast' },
  ],
  cases: [
    { id: 'normal', label: 'Normal', title: 'Ölçüm hazır', detail: 'Hedef 19 °C' },
    { id: 'disabled', label: 'Pasif eylem', title: 'Salt okunur', detail: 'Kayıt değiştirilemez' },
    {
      id: 'long',
      label: 'Uzun Türkçe metin',
      title: 'Fermentasyon sıcaklığı için beklenen ölçüm henüz kaydedilmedi',
      detail: 'Son ölçümden sonra kayıtları denetle ve hedef sıcaklığı doğrula.',
    },
    { id: 'dense', label: 'Yoğun görünüm', title: 'FV2', detail: '19 °C hedef' },
    { id: 'error', label: 'Hata', title: 'Ölçüm alınamadı', detail: 'Bağlantıyı denetle.' },
  ],
  retry: 'Yeniden dene',
  loading: 'Yükleniyor',
  empty: 'Kayıt yok',
  unit: '°C',
  value: '19,2',
  alarms: ['Acil', 'Uyarı', 'Bilgi'],
} as const;
