export const languages = [
  { code: 'en', name: 'English' },
  { code: 'zh-CN', name: '简体中文' },
  { code: 'hi', name: 'हिन्दी' },
  { code: 'es', name: 'Español' },
  { code: 'ar', name: 'العربية' },
  { code: 'fr', name: 'Français' },
  { code: 'bn', name: 'বাংলা' },
  { code: 'pt-BR', name: 'Português (Brasil)' },
  { code: 'ru', name: 'Русский' },
  { code: 'id', name: 'Bahasa Indonesia' },
  { code: 'de', name: 'Deutsch' },
  { code: 'ja', name: '日本語' },
] as const

export type Language = typeof languages[number]['code']

const ru = {
  tabMode: 'Режим', tabStatus: 'Статус', tabSettings: 'Настройки',
  searching: 'поиск', connecting: 'подключение', connected: 'подключено', error: 'ошибка',
  searchingHint: 'Поиск устройства…', connectingHint: 'Ожидание потока модуля…', connectedHint: 'Поток сенсоров активен', errorHint: 'Ошибка подключения',
  left: 'Левый', right: 'Правый', online: 'на связи', offline: 'не в сети', battery: 'Батарея',
  modeGroup: 'Режим панели', touchpad: 'Тачпад', position: 'позиция', button: 'Кнопка', wholePad: 'вся панель', twoButtons: '2 кнопки', upperLower: 'верх / низ',
  touchpadNote: 'Положение и усилие без виртуальных кнопок', pressStrength: 'Сила нажатия', releaseAt: 'Отпускание при 0.20', hapticPower: 'Мощность вибрации',
  force: 'УСИЛИЕ', upper: 'ВЕРХ', lower: 'НИЗ', threshold: 'Порог',
  bundledAdb: 'встроенный ADB', autoDiscovery: 'автообнаружение', moduleRequired: 'Нужен запущенный qptp-module на Quest Pro',
  language: 'Язык', steamvrLifecycle: 'Запускать и закрывать вместе со SteamVR', saveError: 'Не удалось сохранить настройку', openError: 'Не удалось открыть ссылку',
} as const

export type TranslationKey = keyof typeof ru
type Messages = Record<TranslationKey, string>

const strings: Record<Language, Messages> = {
  ru,
  en: {
    tabMode: 'Mode', tabStatus: 'Status', tabSettings: 'Settings',
    searching: 'searching', connecting: 'connecting', connected: 'connected', error: 'error',
    searchingHint: 'Searching for the device…', connectingHint: 'Waiting for the module stream…', connectedHint: 'Sensor stream active', errorHint: 'Connection error',
    left: 'Left', right: 'Right', online: 'connected', offline: 'offline', battery: 'Battery',
    modeGroup: 'Pad mode', touchpad: 'Touchpad', position: 'position', button: 'Button', wholePad: 'whole pad', twoButtons: '2 buttons', upperLower: 'top / bottom',
    touchpadNote: 'Position and force without virtual buttons', pressStrength: 'Press force', releaseAt: 'Release at 0.20', hapticPower: 'Vibration strength',
    force: 'FORCE', upper: 'TOP', lower: 'BOTTOM', threshold: 'Threshold',
    bundledAdb: 'bundled ADB', autoDiscovery: 'auto discovery', moduleRequired: 'qptp-module must run on Quest Pro',
    language: 'Language', steamvrLifecycle: 'Start and close with SteamVR', saveError: 'Could not save the setting', openError: 'Could not open the link',
  },
  'zh-CN': {
    tabMode: '模式', tabStatus: '状态', tabSettings: '设置',
    searching: '搜索中', connecting: '连接中', connected: '已连接', error: '错误',
    searchingHint: '正在搜索设备…', connectingHint: '等待模块数据流…', connectedHint: '传感器数据流已启用', errorHint: '连接错误',
    left: '左手', right: '右手', online: '已连接', offline: '离线', battery: '电量',
    modeGroup: '触控板模式', touchpad: '触控板', position: '位置', button: '按钮', wholePad: '整个面板', twoButtons: '双按钮', upperLower: '上 / 下',
    touchpadNote: '仅使用位置和压力，无虚拟按钮', pressStrength: '按压力度', releaseAt: '低于 0.20 时松开', hapticPower: '振动强度',
    force: '压力', upper: '上', lower: '下', threshold: '阈值',
    bundledAdb: '内置 ADB', autoDiscovery: '自动发现', moduleRequired: '请在 Quest Pro 上运行 qptp-module',
    language: '语言', steamvrLifecycle: '随 SteamVR 启动和关闭', saveError: '无法保存设置', openError: '无法打开链接',
  },
  hi: {
    tabMode: 'मोड', tabStatus: 'स्थिति', tabSettings: 'सेटिंग्स',
    searching: 'खोज जारी', connecting: 'कनेक्ट हो रहा', connected: 'कनेक्टेड', error: 'त्रुटि',
    searchingHint: 'डिवाइस खोजा जा रहा है…', connectingHint: 'मॉड्यूल स्ट्रीम की प्रतीक्षा…', connectedHint: 'सेंसर स्ट्रीम सक्रिय है', errorHint: 'कनेक्शन त्रुटि',
    left: 'बायाँ', right: 'दायाँ', online: 'कनेक्टेड', offline: 'ऑफ़लाइन', battery: 'बैटरी',
    modeGroup: 'पैड मोड', touchpad: 'टचपैड', position: 'स्थिति', button: 'बटन', wholePad: 'पूरा पैड', twoButtons: '2 बटन', upperLower: 'ऊपर / नीचे',
    touchpadNote: 'बिना वर्चुअल बटन के स्थिति और दबाव', pressStrength: 'दबाने का बल', releaseAt: '0.20 पर छोड़ें', hapticPower: 'कंपन की तीव्रता',
    force: 'दबाव', upper: 'ऊपर', lower: 'नीचे', threshold: 'सीमा',
    bundledAdb: 'अंतर्निहित ADB', autoDiscovery: 'स्वतः खोज', moduleRequired: 'Quest Pro पर qptp-module चालू होना चाहिए',
    language: 'भाषा', steamvrLifecycle: 'SteamVR के साथ शुरू और बंद करें', saveError: 'सेटिंग सहेजी नहीं जा सकी', openError: 'लिंक नहीं खुल सका',
  },
  es: {
    tabMode: 'Modo', tabStatus: 'Estado', tabSettings: 'Ajustes',
    searching: 'buscando', connecting: 'conectando', connected: 'conectado', error: 'error',
    searchingHint: 'Buscando el dispositivo…', connectingHint: 'Esperando datos del módulo…', connectedHint: 'Flujo de sensores activo', errorHint: 'Error de conexión',
    left: 'Izquierdo', right: 'Derecho', online: 'conectado', offline: 'sin conexión', battery: 'Batería',
    modeGroup: 'Modo del panel', touchpad: 'Panel táctil', position: 'posición', button: 'Botón', wholePad: 'todo el panel', twoButtons: '2 botones', upperLower: 'arriba / abajo',
    touchpadNote: 'Posición y presión sin botones virtuales', pressStrength: 'Fuerza de pulsación', releaseAt: 'Soltar a 0.20', hapticPower: 'Intensidad de vibración',
    force: 'FUERZA', upper: 'ARRIBA', lower: 'ABAJO', threshold: 'Umbral',
    bundledAdb: 'ADB incluido', autoDiscovery: 'detección automática', moduleRequired: 'qptp-module debe ejecutarse en Quest Pro',
    language: 'Idioma', steamvrLifecycle: 'Iniciar y cerrar con SteamVR', saveError: 'No se pudo guardar el ajuste', openError: 'No se pudo abrir el enlace',
  },
  ar: {
    tabMode: 'الوضع', tabStatus: 'الحالة', tabSettings: 'الإعدادات',
    searching: 'جارٍ البحث', connecting: 'جارٍ الاتصال', connected: 'متصل', error: 'خطأ',
    searchingHint: 'جارٍ البحث عن الجهاز…', connectingHint: 'بانتظار بث الوحدة…', connectedHint: 'بث المستشعرات نشط', errorHint: 'خطأ في الاتصال',
    left: 'الأيسر', right: 'الأيمن', online: 'متصل', offline: 'غير متصل', battery: 'البطارية',
    modeGroup: 'وضع اللوحة', touchpad: 'لوحة اللمس', position: 'الموضع', button: 'زر', wholePad: 'اللوحة كاملة', twoButtons: 'زران', upperLower: 'أعلى / أسفل',
    touchpadNote: 'الموضع والضغط دون أزرار افتراضية', pressStrength: 'قوة الضغط', releaseAt: 'التحرير عند 0.20', hapticPower: 'شدة الاهتزاز',
    force: 'الضغط', upper: 'أعلى', lower: 'أسفل', threshold: 'الحد',
    bundledAdb: 'ADB مدمج', autoDiscovery: 'اكتشاف تلقائي', moduleRequired: 'يجب تشغيل qptp-module على Quest Pro',
    language: 'اللغة', steamvrLifecycle: 'التشغيل والإغلاق مع SteamVR', saveError: 'تعذر حفظ الإعداد', openError: 'تعذر فتح الرابط',
  },
  fr: {
    tabMode: 'Mode', tabStatus: 'État', tabSettings: 'Paramètres',
    searching: 'recherche', connecting: 'connexion', connected: 'connecté', error: 'erreur',
    searchingHint: 'Recherche de l’appareil…', connectingHint: 'Attente du flux du module…', connectedHint: 'Flux des capteurs actif', errorHint: 'Erreur de connexion',
    left: 'Gauche', right: 'Droit', online: 'connecté', offline: 'hors ligne', battery: 'Batterie',
    modeGroup: 'Mode du pavé', touchpad: 'Pavé tactile', position: 'position', button: 'Bouton', wholePad: 'pavé entier', twoButtons: '2 boutons', upperLower: 'haut / bas',
    touchpadNote: 'Position et pression sans boutons virtuels', pressStrength: 'Force d’appui', releaseAt: 'Relâchement à 0.20', hapticPower: 'Intensité des vibrations',
    force: 'PRESSION', upper: 'HAUT', lower: 'BAS', threshold: 'Seuil',
    bundledAdb: 'ADB intégré', autoDiscovery: 'détection auto', moduleRequired: 'qptp-module doit tourner sur Quest Pro',
    language: 'Langue', steamvrLifecycle: 'Démarrer et fermer avec SteamVR', saveError: 'Impossible d’enregistrer le réglage', openError: 'Impossible d’ouvrir le lien',
  },
  bn: {
    tabMode: 'মোড', tabStatus: 'অবস্থা', tabSettings: 'সেটিংস',
    searching: 'খোঁজা হচ্ছে', connecting: 'সংযোগ হচ্ছে', connected: 'সংযুক্ত', error: 'ত্রুটি',
    searchingHint: 'ডিভাইস খোঁজা হচ্ছে…', connectingHint: 'মডিউলের স্ট্রিমের অপেক্ষায়…', connectedHint: 'সেন্সর স্ট্রিম চালু আছে', errorHint: 'সংযোগের ত্রুটি',
    left: 'বাম', right: 'ডান', online: 'সংযুক্ত', offline: 'অফলাইন', battery: 'ব্যাটারি',
    modeGroup: 'প্যাড মোড', touchpad: 'টাচপ্যাড', position: 'অবস্থান', button: 'বোতাম', wholePad: 'পুরো প্যাড', twoButtons: '২ বোতাম', upperLower: 'উপরে / নিচে',
    touchpadNote: 'ভার্চুয়াল বোতাম ছাড়া অবস্থান ও চাপ', pressStrength: 'চাপের মাত্রা', releaseAt: '0.20-তে ছাড়ুন', hapticPower: 'কম্পনের মাত্রা',
    force: 'চাপ', upper: 'উপরে', lower: 'নিচে', threshold: 'সীমা',
    bundledAdb: 'অন্তর্ভুক্ত ADB', autoDiscovery: 'স্বয়ংক্রিয় খোঁজ', moduleRequired: 'Quest Pro-তে qptp-module চালু থাকতে হবে',
    language: 'ভাষা', steamvrLifecycle: 'SteamVR-এর সাথে চালু ও বন্ধ করুন', saveError: 'সেটিং সংরক্ষণ করা যায়নি', openError: 'লিংক খোলা যায়নি',
  },
  'pt-BR': {
    tabMode: 'Modo', tabStatus: 'Status', tabSettings: 'Configurações',
    searching: 'buscando', connecting: 'conectando', connected: 'conectado', error: 'erro',
    searchingHint: 'Buscando o dispositivo…', connectingHint: 'Aguardando o fluxo do módulo…', connectedHint: 'Fluxo dos sensores ativo', errorHint: 'Erro de conexão',
    left: 'Esquerdo', right: 'Direito', online: 'conectado', offline: 'offline', battery: 'Bateria',
    modeGroup: 'Modo do painel', touchpad: 'Touchpad', position: 'posição', button: 'Botão', wholePad: 'painel inteiro', twoButtons: '2 botões', upperLower: 'cima / baixo',
    touchpadNote: 'Posição e pressão sem botões virtuais', pressStrength: 'Força do toque', releaseAt: 'Soltar em 0.20', hapticPower: 'Intensidade da vibração',
    force: 'FORÇA', upper: 'CIMA', lower: 'BAIXO', threshold: 'Limite',
    bundledAdb: 'ADB incluído', autoDiscovery: 'descoberta auto', moduleRequired: 'qptp-module deve estar ativo no Quest Pro',
    language: 'Idioma', steamvrLifecycle: 'Iniciar e fechar com o SteamVR', saveError: 'Não foi possível salvar a opção', openError: 'Não foi possível abrir o link',
  },
  id: {
    tabMode: 'Mode', tabStatus: 'Status', tabSettings: 'Pengaturan',
    searching: 'mencari', connecting: 'menghubungkan', connected: 'terhubung', error: 'galat',
    searchingHint: 'Mencari perangkat…', connectingHint: 'Menunggu aliran modul…', connectedHint: 'Aliran sensor aktif', errorHint: 'Kesalahan koneksi',
    left: 'Kiri', right: 'Kanan', online: 'terhubung', offline: 'offline', battery: 'Baterai',
    modeGroup: 'Mode panel', touchpad: 'Panel sentuh', position: 'posisi', button: 'Tombol', wholePad: 'seluruh panel', twoButtons: '2 tombol', upperLower: 'atas / bawah',
    touchpadNote: 'Posisi dan tekanan tanpa tombol virtual', pressStrength: 'Tekanan tekan', releaseAt: 'Lepas pada 0.20', hapticPower: 'Kekuatan getaran',
    force: 'TEKANAN', upper: 'ATAS', lower: 'BAWAH', threshold: 'Ambang',
    bundledAdb: 'ADB bawaan', autoDiscovery: 'temukan otomatis', moduleRequired: 'qptp-module harus berjalan di Quest Pro',
    language: 'Bahasa', steamvrLifecycle: 'Mulai dan tutup bersama SteamVR', saveError: 'Pengaturan tidak dapat disimpan', openError: 'Tautan tidak dapat dibuka',
  },
  ja: {
    tabMode: 'モード', tabStatus: '状態', tabSettings: '設定',
    searching: '検索中', connecting: '接続中', connected: '接続済み', error: 'エラー',
    searchingHint: 'デバイスを検索中…', connectingHint: 'モジュールのデータを待機中…', connectedHint: 'センサーデータを受信中', errorHint: '接続エラー',
    left: '左', right: '右', online: '接続中', offline: 'オフライン', battery: 'バッテリー',
    modeGroup: 'パッドのモード', touchpad: 'タッチパッド', position: '位置', button: 'ボタン', wholePad: '全体', twoButtons: '2ボタン', upperLower: '上 / 下',
    touchpadNote: '仮想ボタンを使わず位置と圧力を表示', pressStrength: '押す強さ', releaseAt: '0.20 で解除', hapticPower: '振動の強さ',
    force: '圧力', upper: '上', lower: '下', threshold: 'しきい値',
    bundledAdb: '内蔵 ADB', autoDiscovery: '自動検出', moduleRequired: 'Quest Pro で qptp-module を起動してください',
    language: '言語', steamvrLifecycle: 'SteamVR と一緒に起動・終了', saveError: '設定を保存できませんでした', openError: 'リンクを開けませんでした',
  },
  de: {
    tabMode: 'Modus', tabStatus: 'Status', tabSettings: 'Einstellungen',
    searching: 'suche', connecting: 'verbinde', connected: 'verbunden', error: 'Fehler',
    searchingHint: 'Gerät wird gesucht…', connectingHint: 'Warte auf den Modulstream…', connectedHint: 'Sensorstream aktiv', errorHint: 'Verbindungsfehler',
    left: 'Links', right: 'Rechts', online: 'verbunden', offline: 'offline', battery: 'Akku',
    modeGroup: 'Pad-Modus', touchpad: 'Touchpad', position: 'Position', button: 'Taste', wholePad: 'gesamtes Pad', twoButtons: '2 Tasten', upperLower: 'oben / unten',
    touchpadNote: 'Position und Druck ohne virtuelle Tasten', pressStrength: 'Druckstärke', releaseAt: 'Loslassen bei 0.20', hapticPower: 'Vibrationsstärke',
    force: 'DRUCK', upper: 'OBEN', lower: 'UNTEN', threshold: 'Schwelle',
    bundledAdb: 'integriertes ADB', autoDiscovery: 'automatische Suche', moduleRequired: 'qptp-module muss auf Quest Pro laufen',
    language: 'Sprache', steamvrLifecycle: 'Mit SteamVR starten und beenden', saveError: 'Einstellung konnte nicht gespeichert werden', openError: 'Link konnte nicht geöffnet werden',
  },
}

export function isLanguage(value: string): value is Language {
  return languages.some(language => language.code === value)
}

export function detectLanguage(locales: readonly string[]): Language {
  for (const value of locales) {
    const locale = value.trim().replaceAll('_', '-').toLowerCase()
    const primary = locale.split('-')[0] || ''
    if (primary === 'zh') {
      if (locale === 'zh' || locale.startsWith('zh-cn') || locale.startsWith('zh-sg') || locale.startsWith('zh-hans')) return 'zh-CN'
      continue
    }
    if (primary === 'pt') return 'pt-BR'
    if (primary === 'in') return 'id'
    if (isLanguage(primary)) return primary
  }
  return 'en'
}

export function translate(language: Language, key: TranslationKey): string {
  return strings[language][key]
}
