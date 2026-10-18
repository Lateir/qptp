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
  tabSensors: 'Датчики', tabMode: 'Тачпады', tabStatus: 'Статус', tabSettings: 'Настройки',
    stylusSensor: 'Стилус', triggerProximitySensor: 'Палец у триггера', triggerSlideSensor: 'Сдвиг по триггеру',
  searching: 'поиск', connecting: 'подключение', connected: 'подключено', error: 'ошибка',
  searchingHint: 'Поиск устройства…', connectingHint: 'Ожидание потока модуля…', connectedHint: 'Поток сенсоров активен', errorHint: 'Ошибка подключения',
  left: 'Левый', right: 'Правый', online: 'Подключено', offline: 'Не подключено', battery: 'Батарея',
  modeGroup: 'Режим панели', touchpad: 'Тачпад', position: 'позиция', button: 'Кнопка', wholePad: 'вся панель', twoButtons: '2 кнопки', upperLower: 'верх / низ',
  touchpadNote: 'Положение и усилие без виртуальных кнопок', pressStrength: 'Нажатие', releaseAt: 'Отпускание', hapticPower: 'Мощность вибрации',
  force: 'УСИЛИЕ', upper: 'ВЕРХ', lower: 'НИЗ', threshold: 'Порог',
  bundledAdb: 'встроенный ADB', autoDiscovery: 'автообнаружение', moduleRequired: 'Нужен запущенный qptp-module на Quest Pro',
    updateRequired: "Обновите модуль Quest Pro Touch Plus", magiskUpdateHint: "Можно обновить в Magisk: нажмите «Обновить» на модуле", updateAvailable: "Доступно обновление", moduleLabel: "Модуль", appLabel: "Приложение", compatibleReleasePending: "Совместимый релиз пока не найден",
  language: 'Язык', steamvrLifecycle: 'Запускать и закрывать вместе со SteamVR', saveError: 'Не удалось сохранить настройку', openError: 'Не удалось открыть ссылку',
  styleGroup: "Стиль тачпада", styleAbsolute: "Позиция", styleJoystick: "Джойстик", styleSwipe: "Свайп", styleMouse: "Мышь", styleAbsoluteNote: "Точка касания как есть", styleJoystickNote: "Отклонение от места касания", styleSwipeNote: "Скорость движения с инерцией", styleMouseNote: "Курсор Windows, клик — левая кнопка", joystickRange: "Радиус джойстика", swipeGain: "Чувствительность", mouseSpeed: "Скорость курсора", clickForce: "Клик нажатием", palmRejection: "Игнорировать лежащий палец", palmNeedsModule: "нужен модуль v3.6+", calibrate: "Калибровка", calibrationTitle: "Калибровка тачпадов", calibrationStep: "Шаг", calibrationEdges: "Обведите пальцем края обеих панелей", calibrationUp: "Проведите пальцем прямо вверх на каждой панели", calibrationForce: "Коснитесь легко, затем нажмите изо всех сил", back: "Назад", next: "Далее", skip: "Пропустить", save: "Сохранить", reset: "Сбросить", cancel: "Отмена", steamvrDriver: "Драйвер SteamVR", driverClassic: "Классический", driverClassicNote: "отдельное устройство", driverFusionNote: "прямо в контроллерах", driverOffline: "SteamVR не запущен", driverRestart: "Перезапустите SteamVR, чтобы применить", driverFallback: "Fusion не включился, работает классический", driverConflict: "Обнаружен QFTPlus — отключите его тачпад", driverClassicActive: "Работает классический режим", driverFusionActive: "Fusion активен", driverReconnect: "переподключите контроллер",
  quest3Title: "Это контроллеры Quest Pro?", quest3Body: "SteamVR видит контроллеры Quest 3 — так бывает при подключении через Virtual Desktop. Если это Quest Pro, Fusion добавит в них тачпад и покажет модель Quest Pro после перезапуска SteamVR.", quest3Yes: "Да, это Quest Pro", quest3No: "Нет", quest3Skipped: "Контроллеры Quest 3 пропущены", change: "изменить",
} as const

export type TranslationKey = keyof typeof ru
type Messages = Record<TranslationKey, string>

const strings: Record<Language, Messages> = {
  ru,
  en: {
    tabSensors: 'Sensors', tabMode: 'Touchpads', tabStatus: 'Status', tabSettings: 'Settings',
    stylusSensor: 'Stylus', triggerProximitySensor: 'Finger near trigger', triggerSlideSensor: 'Trigger surface slide',
    searching: 'searching', connecting: 'connecting', connected: 'connected', error: 'error',
    searchingHint: 'Searching for the device…', connectingHint: 'Waiting for the module stream…', connectedHint: 'Sensor stream active', errorHint: 'Connection error',
    left: 'Left', right: 'Right', online: 'Connected', offline: 'Disconnected', battery: 'Battery',
    modeGroup: 'Pad mode', touchpad: 'Touchpad', position: 'position', button: 'Button', wholePad: 'whole pad', twoButtons: '2 buttons', upperLower: 'top / bottom',
    touchpadNote: 'Position and force without virtual buttons', pressStrength: 'Press', releaseAt: 'Release', hapticPower: 'Vibration strength',
    force: 'FORCE', upper: 'TOP', lower: 'BOTTOM', threshold: 'Threshold',
    bundledAdb: 'bundled ADB', autoDiscovery: 'auto discovery', moduleRequired: 'qptp-module must run on Quest Pro',
    updateRequired: "Update the Quest Pro Touch Plus module", magiskUpdateHint: "You can also tap Update for the module in Magisk", updateAvailable: "Update available", moduleLabel: "Module", appLabel: "App", compatibleReleasePending: "No compatible release has been found yet",
    language: 'Language', steamvrLifecycle: 'Start and exit with SteamVR', saveError: 'Could not save the setting', openError: 'Could not open the link',
    styleGroup: "Touchpad style", styleAbsolute: "Position", styleJoystick: "Joystick", styleSwipe: "Swipe", styleMouse: "Mouse", styleAbsoluteNote: "Touch point as is", styleJoystickNote: "Offset from where you touched", styleSwipeNote: "Movement speed with inertia", styleMouseNote: "Windows cursor, click is left button", joystickRange: "Joystick radius", swipeGain: "Sensitivity", mouseSpeed: "Cursor speed", clickForce: "Press to click", palmRejection: "Ignore a resting thumb", palmNeedsModule: "needs module v3.6+", calibrate: "Calibrate", calibrationTitle: "Touchpad calibration", calibrationStep: "Step", calibrationEdges: "Trace the edges of both pads", calibrationUp: "Swipe straight up on each pad", calibrationForce: "Touch lightly, then press as hard as you can", back: "Back", next: "Next", skip: "Skip", save: "Save", reset: "Reset", cancel: "Cancel", steamvrDriver: "SteamVR driver", driverClassic: "Classic", driverClassicNote: "separate device", driverFusionNote: "inside the controllers", driverOffline: "SteamVR is not running", driverRestart: "Restart SteamVR to apply", driverFallback: "Fusion did not start; Classic is running", driverConflict: "QFTPlus detected: turn off its touchpad", driverClassicActive: "Classic mode is running", driverFusionActive: "Fusion is active", driverReconnect: "reconnect the controller",
    quest3Title: "Are these Quest Pro controllers?", quest3Body: "SteamVR sees Quest 3 controllers, which can happen through Virtual Desktop. If they are Quest Pro, Fusion adds the touchpad to them and shows the Quest Pro model after SteamVR restarts.", quest3Yes: "Yes, Quest Pro", quest3No: "No", quest3Skipped: "Quest 3 controllers skipped", change: "change",
  },
  'zh-CN': {
    tabSensors: '传感器', tabMode: '触控板', tabStatus: '状态', tabSettings: '设置',
    stylusSensor: '触控笔', triggerProximitySensor: '扳机附近的手指', triggerSlideSensor: '扳机表面滑动',
    searching: '搜索中', connecting: '连接中', connected: '已连接', error: '错误',
    searchingHint: '正在搜索设备…', connectingHint: '等待模块数据流…', connectedHint: '传感器数据流已启用', errorHint: '连接错误',
    left: '左手', right: '右手', online: '已连接', offline: '未连接', battery: '电量',
    modeGroup: '触控板模式', touchpad: '触控板', position: '位置', button: '按钮', wholePad: '整个面板', twoButtons: '双按钮', upperLower: '上 / 下',
    touchpadNote: '仅使用位置和压力，无虚拟按钮', pressStrength: '按下', releaseAt: '松开', hapticPower: '振动强度',
    force: '压力', upper: '上', lower: '下', threshold: '阈值',
    bundledAdb: '内置 ADB', autoDiscovery: '自动发现', moduleRequired: '请在 Quest Pro 上运行 qptp-module',
    updateRequired: "更新 Quest Pro Touch Plus 模块", magiskUpdateHint: "也可在 Magisk 中点击模块的“更新”", updateAvailable: "有可用更新", moduleLabel: "模块", appLabel: "应用", compatibleReleasePending: "尚未找到兼容版本",
    language: '语言', steamvrLifecycle: '随 SteamVR 启动和关闭', saveError: '无法保存设置', openError: '无法打开链接',
    styleGroup: "触控板样式", styleAbsolute: "位置", styleJoystick: "摇杆", styleSwipe: "滑动", styleMouse: "鼠标", styleAbsoluteNote: "按触点原样输出", styleJoystickNote: "相对触点的偏移", styleSwipeNote: "带惯性的移动速度", styleMouseNote: "Windows 光标，点击为左键", joystickRange: "摇杆半径", swipeGain: "灵敏度", mouseSpeed: "光标速度", clickForce: "按压点击", palmRejection: "忽略搭在上面的拇指", palmNeedsModule: "需要模块 v3.6+", calibrate: "校准", calibrationTitle: "触控板校准", calibrationStep: "步骤", calibrationEdges: "用手指沿两个触控板的边缘划一圈", calibrationUp: "在每个触控板上笔直向上滑动", calibrationForce: "先轻触，再用力按到底", back: "返回", next: "下一步", skip: "跳过", save: "保存", reset: "重置", cancel: "取消", steamvrDriver: "SteamVR 驱动", driverClassic: "经典", driverClassicNote: "独立设备", driverFusionNote: "直接集成到控制器", driverOffline: "SteamVR 未运行", driverRestart: "重启 SteamVR 以生效", driverFallback: "Fusion 未能启动，正在使用经典模式", driverConflict: "检测到 QFTPlus：请关闭它的触控板", driverClassicActive: "经典模式运行中", driverFusionActive: "Fusion 已启用", driverReconnect: "请重新连接控制器",
    quest3Title: "这是 Quest Pro 控制器吗？", quest3Body: "SteamVR 识别到的是 Quest 3 控制器，通过 Virtual Desktop 连接时可能会这样。如果它们其实是 Quest Pro，重启 SteamVR 后 Fusion 会为其添加触控板并显示 Quest Pro 模型。", quest3Yes: "是，Quest Pro", quest3No: "否", quest3Skipped: "已跳过 Quest 3 控制器", change: "更改",
  },
  hi: {
    tabSensors: 'सेंसर', tabMode: 'टचपैड', tabStatus: 'स्थिति', tabSettings: 'सेटिंग्स',
    stylusSensor: 'स्टाइलस', triggerProximitySensor: 'ट्रिगर के पास उंगली', triggerSlideSensor: 'ट्रिगर पर उंगली सरकाना',
    searching: 'खोज जारी', connecting: 'कनेक्ट हो रहा', connected: 'कनेक्टेड', error: 'त्रुटि',
    searchingHint: 'डिवाइस खोजा जा रहा है…', connectingHint: 'मॉड्यूल स्ट्रीम की प्रतीक्षा…', connectedHint: 'सेंसर स्ट्रीम सक्रिय है', errorHint: 'कनेक्शन त्रुटि',
    left: 'बायाँ', right: 'दायाँ', online: 'कनेक्टेड', offline: 'कनेक्ट नहीं है', battery: 'बैटरी',
    modeGroup: 'पैड मोड', touchpad: 'टचपैड', position: 'स्थिति', button: 'बटन', wholePad: 'पूरा पैड', twoButtons: '2 बटन', upperLower: 'ऊपर / नीचे',
    touchpadNote: 'बिना वर्चुअल बटन के स्थिति और दबाव', pressStrength: 'दबाएँ', releaseAt: 'छोड़ें', hapticPower: 'कंपन की तीव्रता',
    force: 'दबाव', upper: 'ऊपर', lower: 'नीचे', threshold: 'सीमा',
    bundledAdb: 'अंतर्निहित ADB', autoDiscovery: 'स्वतः खोज', moduleRequired: 'Quest Pro पर qptp-module चालू होना चाहिए',
    updateRequired: "Quest Pro Touch Plus मॉड्यूल अपडेट करें", magiskUpdateHint: "Magisk में मॉड्यूल पर अपडेट भी दबा सकते हैं", updateAvailable: "अपडेट उपलब्ध है", moduleLabel: "मॉड्यूल", appLabel: "ऐप", compatibleReleasePending: "संगत रिलीज़ अभी नहीं मिली है",
    language: 'भाषा', steamvrLifecycle: 'SteamVR के साथ शुरू और बंद करें', saveError: 'सेटिंग सहेजी नहीं जा सकी', openError: 'लिंक नहीं खुल सका',
    styleGroup: "टचपैड शैली", styleAbsolute: "स्थिति", styleJoystick: "जॉयस्टिक", styleSwipe: "स्वाइप", styleMouse: "माउस", styleAbsoluteNote: "छूने का बिंदु जैसा है", styleJoystickNote: "छूने की जगह से विचलन", styleSwipeNote: "जड़त्व के साथ गति", styleMouseNote: "Windows कर्सर, क्लिक बायाँ बटन", joystickRange: "जॉयस्टिक त्रिज्या", swipeGain: "संवेदनशीलता", mouseSpeed: "कर्सर गति", clickForce: "दबाकर क्लिक", palmRejection: "टिके अंगूठे को अनदेखा करें", palmNeedsModule: "मॉड्यूल v3.6+ चाहिए", calibrate: "कैलिब्रेट", calibrationTitle: "टचपैड कैलिब्रेशन", calibrationStep: "चरण", calibrationEdges: "दोनों पैड के किनारों पर उंगली घुमाएँ", calibrationUp: "हर पैड पर सीधे ऊपर स्वाइप करें", calibrationForce: "हल्के से छुएँ, फिर पूरी ताकत से दबाएँ", back: "पीछे", next: "आगे", skip: "छोड़ें", save: "सहेजें", reset: "रीसेट", cancel: "रद्द करें", steamvrDriver: "SteamVR ड्राइवर", driverClassic: "क्लासिक", driverClassicNote: "अलग डिवाइस", driverFusionNote: "सीधे कंट्रोलर में", driverOffline: "SteamVR नहीं चल रहा", driverRestart: "लागू करने के लिए SteamVR फिर से शुरू करें", driverFallback: "Fusion शुरू नहीं हुआ, क्लासिक चल रहा है", driverConflict: "QFTPlus मिला: उसका टचपैड बंद करें", driverClassicActive: "क्लासिक मोड चल रहा है", driverFusionActive: "Fusion सक्रिय है", driverReconnect: "कंट्रोलर फिर से जोड़ें",
    quest3Title: "क्या ये Quest Pro कंट्रोलर हैं?", quest3Body: "SteamVR इन्हें Quest 3 कंट्रोलर के रूप में देखता है, Virtual Desktop से ऐसा हो सकता है। अगर ये Quest Pro हैं, तो SteamVR फिर से शुरू करने के बाद Fusion इनमें टचपैड जोड़ेगा और Quest Pro मॉडल दिखाएगा।", quest3Yes: "हाँ, Quest Pro", quest3No: "नहीं", quest3Skipped: "Quest 3 कंट्रोलर छोड़े गए", change: "बदलें",
  },
  es: {
    tabSensors: 'Sensores', tabMode: 'Paneles táctiles', tabStatus: 'Estado', tabSettings: 'Ajustes',
    stylusSensor: 'Lápiz', triggerProximitySensor: 'Dedo cerca del gatillo', triggerSlideSensor: 'Deslizamiento en el gatillo',
    searching: 'buscando', connecting: 'conectando', connected: 'conectado', error: 'error',
    searchingHint: 'Buscando el dispositivo…', connectingHint: 'Esperando datos del módulo…', connectedHint: 'Flujo de sensores activo', errorHint: 'Error de conexión',
    left: 'Izquierdo', right: 'Derecho', online: 'Conectado', offline: 'Desconectado', battery: 'Batería',
    modeGroup: 'Modo del panel', touchpad: 'Panel táctil', position: 'posición', button: 'Botón', wholePad: 'todo el panel', twoButtons: '2 botones', upperLower: 'arriba / abajo',
    touchpadNote: 'Posición y presión sin botones virtuales', pressStrength: 'Pulsar', releaseAt: 'Soltar', hapticPower: 'Intensidad de vibración',
    force: 'FUERZA', upper: 'ARRIBA', lower: 'ABAJO', threshold: 'Umbral',
    bundledAdb: 'ADB incluido', autoDiscovery: 'detección automática', moduleRequired: 'qptp-module debe ejecutarse en Quest Pro',
    updateRequired: "Actualiza el módulo Quest Pro Touch Plus", magiskUpdateHint: "También puedes pulsar Actualizar en el módulo de Magisk", updateAvailable: "Actualización disponible", moduleLabel: "Módulo", appLabel: "Aplicación", compatibleReleasePending: "Aún no se ha encontrado una versión compatible",
    language: 'Idioma', steamvrLifecycle: 'Iniciar y cerrar con SteamVR', saveError: 'No se pudo guardar el ajuste', openError: 'No se pudo abrir el enlace',
    styleGroup: "Estilo del touchpad", styleAbsolute: "Posición", styleJoystick: "Joystick", styleSwipe: "Deslizar", styleMouse: "Ratón", styleAbsoluteNote: "El punto de contacto tal cual", styleJoystickNote: "Desplazamiento desde donde tocas", styleSwipeNote: "Velocidad del movimiento con inercia", styleMouseNote: "Cursor de Windows, clic con botón izquierdo", joystickRange: "Radio del joystick", swipeGain: "Sensibilidad", mouseSpeed: "Velocidad del cursor", clickForce: "Pulsar para clic", palmRejection: "Ignorar el pulgar apoyado", palmNeedsModule: "requiere módulo v3.6+", calibrate: "Calibrar", calibrationTitle: "Calibración de touchpads", calibrationStep: "Paso", calibrationEdges: "Recorre con el dedo los bordes de ambos paneles", calibrationUp: "Desliza recto hacia arriba en cada panel", calibrationForce: "Toca suavemente y luego presiona con toda tu fuerza", back: "Atrás", next: "Siguiente", skip: "Omitir", save: "Guardar", reset: "Restablecer", cancel: "Cancelar", steamvrDriver: "Controlador de SteamVR", driverClassic: "Clásico", driverClassicNote: "dispositivo aparte", driverFusionNote: "dentro de los mandos", driverOffline: "SteamVR no se está ejecutando", driverRestart: "Reinicia SteamVR para aplicarlo", driverFallback: "Fusion no se inició; funciona el clásico", driverConflict: "QFTPlus detectado: desactiva su touchpad", driverClassicActive: "Modo clásico en uso", driverFusionActive: "Fusion activo", driverReconnect: "vuelve a conectar el mando",
    quest3Title: "¿Son mandos Quest Pro?", quest3Body: "SteamVR detecta mandos Quest 3, algo que puede pasar con Virtual Desktop. Si son Quest Pro, Fusion les añadirá el touchpad y mostrará el modelo Quest Pro tras reiniciar SteamVR.", quest3Yes: "Sí, Quest Pro", quest3No: "No", quest3Skipped: "Mandos Quest 3 omitidos", change: "cambiar",
  },
  ar: {
    tabSensors: 'المستشعرات', tabMode: 'لوحات اللمس', tabStatus: 'الحالة', tabSettings: 'الإعدادات',
    stylusSensor: 'القلم', triggerProximitySensor: 'الإصبع قرب الزناد', triggerSlideSensor: 'انزلاق الإصبع على الزناد',
    searching: 'جارٍ البحث', connecting: 'جارٍ الاتصال', connected: 'متصل', error: 'خطأ',
    searchingHint: 'جارٍ البحث عن الجهاز…', connectingHint: 'بانتظار بث الوحدة…', connectedHint: 'بث المستشعرات نشط', errorHint: 'خطأ في الاتصال',
    left: 'الأيسر', right: 'الأيمن', online: 'متصل', offline: 'غير متصل', battery: 'البطارية',
    modeGroup: 'وضع اللوحة', touchpad: 'لوحة اللمس', position: 'الموضع', button: 'زر', wholePad: 'اللوحة كاملة', twoButtons: 'زران', upperLower: 'أعلى / أسفل',
    touchpadNote: 'الموضع والضغط دون أزرار افتراضية', pressStrength: 'الضغط', releaseAt: 'التحرير', hapticPower: 'شدة الاهتزاز',
    force: 'الضغط', upper: 'أعلى', lower: 'أسفل', threshold: 'الحد',
    bundledAdb: 'ADB مدمج', autoDiscovery: 'اكتشاف تلقائي', moduleRequired: 'يجب تشغيل qptp-module على Quest Pro',
    updateRequired: "حدّث وحدة Quest Pro Touch Plus", magiskUpdateHint: "يمكنك أيضًا الضغط على تحديث للوحدة في Magisk", updateAvailable: "يتوفر تحديث", moduleLabel: "الوحدة", appLabel: "التطبيق", compatibleReleasePending: "لم يُعثر بعد على إصدار متوافق",
    language: 'اللغة', steamvrLifecycle: 'التشغيل والإغلاق مع SteamVR', saveError: 'تعذر حفظ الإعداد', openError: 'تعذر فتح الرابط',
    styleGroup: "نمط لوحة اللمس", styleAbsolute: "الموضع", styleJoystick: "عصا تحكم", styleSwipe: "سحب", styleMouse: "ماوس", styleAbsoluteNote: "نقطة اللمس كما هي", styleJoystickNote: "الإزاحة من موضع اللمس", styleSwipeNote: "سرعة الحركة مع القصور الذاتي", styleMouseNote: "مؤشر Windows، النقر بالزر الأيسر", joystickRange: "نصف قطر العصا", swipeGain: "الحساسية", mouseSpeed: "سرعة المؤشر", clickForce: "الضغط للنقر", palmRejection: "تجاهل الإبهام المستريح", palmNeedsModule: "يتطلب الوحدة v3.6+", calibrate: "معايرة", calibrationTitle: "معايرة لوحات اللمس", calibrationStep: "الخطوة", calibrationEdges: "مرّر إصبعك على حواف اللوحتين", calibrationUp: "اسحب للأعلى بشكل مستقيم على كل لوحة", calibrationForce: "المس بخفة، ثم اضغط بأقصى قوة", back: "رجوع", next: "التالي", skip: "تخطٍّ", save: "حفظ", reset: "إعادة تعيين", cancel: "إلغاء", steamvrDriver: "برنامج تشغيل SteamVR", driverClassic: "الكلاسيكي", driverClassicNote: "جهاز منفصل", driverFusionNote: "داخل وحدات التحكم", driverOffline: "SteamVR غير مشغّل", driverRestart: "أعد تشغيل SteamVR للتطبيق", driverFallback: "لم يبدأ Fusion؛ يعمل الوضع الكلاسيكي", driverConflict: "تم اكتشاف QFTPlus: أوقف لوحة اللمس الخاصة به", driverClassicActive: "الوضع الكلاسيكي يعمل", driverFusionActive: "Fusion مفعّل", driverReconnect: "أعد توصيل وحدة التحكم",
    quest3Title: "هل هذه وحدات تحكم Quest Pro؟", quest3Body: "يرى SteamVR وحدات تحكم Quest 3، وقد يحدث ذلك عبر Virtual Desktop. إذا كانت Quest Pro، فسيضيف Fusion لوحة اللمس إليها ويعرض طراز Quest Pro بعد إعادة تشغيل SteamVR.", quest3Yes: "نعم، Quest Pro", quest3No: "لا", quest3Skipped: "تم تخطي وحدات تحكم Quest 3", change: "تغيير",
  },
  fr: {
    tabSensors: 'Capteurs', tabMode: 'Pavés tactiles', tabStatus: 'État', tabSettings: 'Paramètres',
    stylusSensor: 'Stylet', triggerProximitySensor: 'Doigt près de la gâchette', triggerSlideSensor: 'Glissement sur la gâchette',
    searching: 'recherche', connecting: 'connexion', connected: 'connecté', error: 'erreur',
    searchingHint: 'Recherche de l’appareil…', connectingHint: 'Attente du flux du module…', connectedHint: 'Flux des capteurs actif', errorHint: 'Erreur de connexion',
    left: 'Gauche', right: 'Droit', online: 'Connecté', offline: 'Déconnecté', battery: 'Batterie',
    modeGroup: 'Mode du pavé', touchpad: 'Pavé tactile', position: 'position', button: 'Bouton', wholePad: 'pavé entier', twoButtons: '2 boutons', upperLower: 'haut / bas',
    touchpadNote: 'Position et pression sans boutons virtuels', pressStrength: 'Appui', releaseAt: 'Relâchement', hapticPower: 'Intensité des vibrations',
    force: 'PRESSION', upper: 'HAUT', lower: 'BAS', threshold: 'Seuil',
    bundledAdb: 'ADB intégré', autoDiscovery: 'détection auto', moduleRequired: 'qptp-module doit tourner sur Quest Pro',
    updateRequired: "Mettez à jour le module Quest Pro Touch Plus", magiskUpdateHint: "Vous pouvez aussi toucher Mettre à jour dans Magisk", updateAvailable: "Mise à jour disponible", moduleLabel: "Module", appLabel: "Application", compatibleReleasePending: "Aucune version compatible trouvée pour le moment",
    language: 'Langue', steamvrLifecycle: 'Démarrer et fermer avec SteamVR', saveError: 'Impossible d’enregistrer le réglage', openError: 'Impossible d’ouvrir le lien',
    styleGroup: "Style du pavé", styleAbsolute: "Position", styleJoystick: "Joystick", styleSwipe: "Balayage", styleMouse: "Souris", styleAbsoluteNote: "Le point de contact tel quel", styleJoystickNote: "Décalage depuis le point touché", styleSwipeNote: "Vitesse du geste avec inertie", styleMouseNote: "Curseur Windows, clic gauche", joystickRange: "Rayon du joystick", swipeGain: "Sensibilité", mouseSpeed: "Vitesse du curseur", clickForce: "Appui pour cliquer", palmRejection: "Ignorer le pouce posé", palmNeedsModule: "module v3.6+ requis", calibrate: "Calibrer", calibrationTitle: "Calibrage des pavés", calibrationStep: "Étape", calibrationEdges: "Suivez du doigt les bords des deux pavés", calibrationUp: "Glissez tout droit vers le haut sur chaque pavé", calibrationForce: "Touchez légèrement, puis appuyez de toutes vos forces", back: "Retour", next: "Suivant", skip: "Passer", save: "Enregistrer", reset: "Réinitialiser", cancel: "Annuler", steamvrDriver: "Pilote SteamVR", driverClassic: "Classique", driverClassicNote: "appareil séparé", driverFusionNote: "dans les manettes", driverOffline: "SteamVR n’est pas lancé", driverRestart: "Redémarrez SteamVR pour appliquer", driverFallback: "Fusion n’a pas démarré ; le mode classique est actif", driverConflict: "QFTPlus détecté : désactivez son pavé tactile", driverClassicActive: "Mode classique actif", driverFusionActive: "Fusion actif", driverReconnect: "reconnectez la manette",
    quest3Title: "Est-ce des manettes Quest Pro ?", quest3Body: "SteamVR voit des manettes Quest 3, ce qui peut arriver avec Virtual Desktop. Si ce sont des Quest Pro, Fusion leur ajoute le pavé tactile et affiche le modèle Quest Pro après le redémarrage de SteamVR.", quest3Yes: "Oui, Quest Pro", quest3No: "Non", quest3Skipped: "Manettes Quest 3 ignorées", change: "modifier",
  },
  bn: {
    tabSensors: 'সেন্সর', tabMode: 'টাচপ্যাড', tabStatus: 'অবস্থা', tabSettings: 'সেটিংস',
    stylusSensor: 'স্টাইলাস', triggerProximitySensor: 'ট্রিগারের কাছে আঙুল', triggerSlideSensor: 'ট্রিগারে আঙুল সরানো',
    searching: 'খোঁজা হচ্ছে', connecting: 'সংযোগ হচ্ছে', connected: 'সংযুক্ত', error: 'ত্রুটি',
    searchingHint: 'ডিভাইস খোঁজা হচ্ছে…', connectingHint: 'মডিউলের স্ট্রিমের অপেক্ষায়…', connectedHint: 'সেন্সর স্ট্রিম চালু আছে', errorHint: 'সংযোগের ত্রুটি',
    left: 'বাম', right: 'ডান', online: 'সংযুক্ত', offline: 'সংযুক্ত নয়', battery: 'ব্যাটারি',
    modeGroup: 'প্যাড মোড', touchpad: 'টাচপ্যাড', position: 'অবস্থান', button: 'বোতাম', wholePad: 'পুরো প্যাড', twoButtons: '২ বোতাম', upperLower: 'উপরে / নিচে',
    touchpadNote: 'ভার্চুয়াল বোতাম ছাড়া অবস্থান ও চাপ', pressStrength: 'চাপুন', releaseAt: 'ছাড়ুন', hapticPower: 'কম্পনের মাত্রা',
    force: 'চাপ', upper: 'উপরে', lower: 'নিচে', threshold: 'সীমা',
    bundledAdb: 'অন্তর্ভুক্ত ADB', autoDiscovery: 'স্বয়ংক্রিয় খোঁজ', moduleRequired: 'Quest Pro-তে qptp-module চালু থাকতে হবে',
    updateRequired: "Quest Pro Touch Plus মডিউল আপডেট করুন", magiskUpdateHint: "Magisk-এ মডিউলের আপডেট বোতামও চাপতে পারেন", updateAvailable: "আপডেট উপলব্ধ", moduleLabel: "মডিউল", appLabel: "অ্যাপ", compatibleReleasePending: "সামঞ্জস্যপূর্ণ রিলিজ এখনো পাওয়া যায়নি",
    language: 'ভাষা', steamvrLifecycle: 'SteamVR-এর সাথে চালু ও বন্ধ করুন', saveError: 'সেটিং সংরক্ষণ করা যায়নি', openError: 'লিংক খোলা যায়নি',
    styleGroup: "টাচপ্যাডের ধরন", styleAbsolute: "অবস্থান", styleJoystick: "জয়স্টিক", styleSwipe: "সোয়াইপ", styleMouse: "মাউস", styleAbsoluteNote: "স্পর্শবিন্দু যেমন আছে", styleJoystickNote: "স্পর্শের জায়গা থেকে সরণ", styleSwipeNote: "জড়তাসহ চলার গতি", styleMouseNote: "Windows কার্সর, ক্লিক বাম বোতাম", joystickRange: "জয়স্টিকের ব্যাসার্ধ", swipeGain: "সংবেদনশীলতা", mouseSpeed: "কার্সরের গতি", clickForce: "চেপে ক্লিক", palmRejection: "রাখা আঙুল উপেক্ষা করুন", palmNeedsModule: "মডিউল v3.6+ প্রয়োজন", calibrate: "ক্যালিব্রেট", calibrationTitle: "টাচপ্যাড ক্যালিব্রেশন", calibrationStep: "ধাপ", calibrationEdges: "দুই প্যাডের প্রান্ত বরাবর আঙুল বোলান", calibrationUp: "প্রতিটি প্যাডে সোজা উপরে সোয়াইপ করুন", calibrationForce: "হালকা ছুঁয়ে, তারপর পুরো জোরে চাপুন", back: "পেছনে", next: "পরবর্তী", skip: "এড়িয়ে যান", save: "সংরক্ষণ", reset: "রিসেট", cancel: "বাতিল", steamvrDriver: "SteamVR ড্রাইভার", driverClassic: "ক্লাসিক", driverClassicNote: "আলাদা ডিভাইস", driverFusionNote: "সরাসরি কন্ট্রোলারে", driverOffline: "SteamVR চলছে না", driverRestart: "প্রয়োগ করতে SteamVR আবার চালু করুন", driverFallback: "Fusion চালু হয়নি, ক্লাসিক চলছে", driverConflict: "QFTPlus পাওয়া গেছে: এর টাচপ্যাড বন্ধ করুন", driverClassicActive: "ক্লাসিক মোড চলছে", driverFusionActive: "Fusion সক্রিয়", driverReconnect: "কন্ট্রোলার আবার সংযোগ করুন",
    quest3Title: "এগুলো কি Quest Pro কন্ট্রোলার?", quest3Body: "SteamVR এগুলোকে Quest 3 কন্ট্রোলার হিসেবে দেখছে, Virtual Desktop দিয়ে এমন হতে পারে। এগুলো Quest Pro হলে SteamVR আবার চালু করার পর Fusion এতে টাচপ্যাড যোগ করবে এবং Quest Pro মডেল দেখাবে।", quest3Yes: "হ্যাঁ, Quest Pro", quest3No: "না", quest3Skipped: "Quest 3 কন্ট্রোলার বাদ দেওয়া হয়েছে", change: "পরিবর্তন",
  },
  'pt-BR': {
    tabSensors: 'Sensores', tabMode: 'Touchpads', tabStatus: 'Status', tabSettings: 'Configurações',
    stylusSensor: 'Caneta', triggerProximitySensor: 'Dedo perto do gatilho', triggerSlideSensor: 'Deslize no gatilho',
    searching: 'buscando', connecting: 'conectando', connected: 'conectado', error: 'erro',
    searchingHint: 'Buscando o dispositivo…', connectingHint: 'Aguardando o fluxo do módulo…', connectedHint: 'Fluxo dos sensores ativo', errorHint: 'Erro de conexão',
    left: 'Esquerdo', right: 'Direito', online: 'Conectado', offline: 'Desconectado', battery: 'Bateria',
    modeGroup: 'Modo do painel', touchpad: 'Touchpad', position: 'posição', button: 'Botão', wholePad: 'painel inteiro', twoButtons: '2 botões', upperLower: 'cima / baixo',
    touchpadNote: 'Posição e pressão sem botões virtuais', pressStrength: 'Pressionar', releaseAt: 'Soltar', hapticPower: 'Intensidade da vibração',
    force: 'FORÇA', upper: 'CIMA', lower: 'BAIXO', threshold: 'Limite',
    bundledAdb: 'ADB incluído', autoDiscovery: 'descoberta auto', moduleRequired: 'qptp-module deve estar ativo no Quest Pro',
    updateRequired: "Atualize o módulo Quest Pro Touch Plus", magiskUpdateHint: "Você também pode tocar em Atualizar no módulo no Magisk", updateAvailable: "Atualização disponível", moduleLabel: "Módulo", appLabel: "Aplicativo", compatibleReleasePending: "Ainda não foi encontrada uma versão compatível",
    language: 'Idioma', steamvrLifecycle: 'Iniciar e fechar com o SteamVR', saveError: 'Não foi possível salvar a opção', openError: 'Não foi possível abrir o link',
    styleGroup: "Estilo do touchpad", styleAbsolute: "Posição", styleJoystick: "Joystick", styleSwipe: "Deslizar", styleMouse: "Mouse", styleAbsoluteNote: "O ponto de toque como está", styleJoystickNote: "Deslocamento a partir do toque", styleSwipeNote: "Velocidade do movimento com inércia", styleMouseNote: "Cursor do Windows, clique com botão esquerdo", joystickRange: "Raio do joystick", swipeGain: "Sensibilidade", mouseSpeed: "Velocidade do cursor", clickForce: "Pressionar para clicar", palmRejection: "Ignorar o polegar apoiado", palmNeedsModule: "requer módulo v3.6+", calibrate: "Calibrar", calibrationTitle: "Calibração dos touchpads", calibrationStep: "Etapa", calibrationEdges: "Contorne com o dedo as bordas dos dois painéis", calibrationUp: "Deslize reto para cima em cada painel", calibrationForce: "Toque de leve e depois pressione com toda a força", back: "Voltar", next: "Avançar", skip: "Pular", save: "Salvar", reset: "Redefinir", cancel: "Cancelar", steamvrDriver: "Driver do SteamVR", driverClassic: "Clássico", driverClassicNote: "dispositivo separado", driverFusionNote: "dentro dos controles", driverOffline: "O SteamVR não está em execução", driverRestart: "Reinicie o SteamVR para aplicar", driverFallback: "O Fusion não iniciou; o clássico está ativo", driverConflict: "QFTPlus detectado: desative o touchpad dele", driverClassicActive: "Modo clássico ativo", driverFusionActive: "Fusion ativo", driverReconnect: "reconecte o controle",
    quest3Title: "Estes são controles Quest Pro?", quest3Body: "O SteamVR vê controles Quest 3, o que pode acontecer pelo Virtual Desktop. Se forem Quest Pro, o Fusion adiciona o touchpad a eles e mostra o modelo Quest Pro depois de reiniciar o SteamVR.", quest3Yes: "Sim, Quest Pro", quest3No: "Não", quest3Skipped: "Controles Quest 3 ignorados", change: "alterar",
  },
  id: {
    tabSensors: 'Sensor', tabMode: 'Panel sentuh', tabStatus: 'Status', tabSettings: 'Pengaturan',
    stylusSensor: 'Stylus', triggerProximitySensor: 'Jari dekat pemicu', triggerSlideSensor: 'Geser di permukaan pemicu',
    searching: 'mencari', connecting: 'menghubungkan', connected: 'terhubung', error: 'galat',
    searchingHint: 'Mencari perangkat…', connectingHint: 'Menunggu aliran modul…', connectedHint: 'Aliran sensor aktif', errorHint: 'Kesalahan koneksi',
    left: 'Kiri', right: 'Kanan', online: 'Terhubung', offline: 'Belum terhubung', battery: 'Baterai',
    modeGroup: 'Mode panel', touchpad: 'Panel sentuh', position: 'posisi', button: 'Tombol', wholePad: 'seluruh panel', twoButtons: '2 tombol', upperLower: 'atas / bawah',
    touchpadNote: 'Posisi dan tekanan tanpa tombol virtual', pressStrength: 'Tekan', releaseAt: 'Lepas', hapticPower: 'Kekuatan getaran',
    force: 'TEKANAN', upper: 'ATAS', lower: 'BAWAH', threshold: 'Ambang',
    bundledAdb: 'ADB bawaan', autoDiscovery: 'temukan otomatis', moduleRequired: 'qptp-module harus berjalan di Quest Pro',
    updateRequired: "Perbarui modul Quest Pro Touch Plus", magiskUpdateHint: "Anda juga dapat mengetuk Perbarui pada modul di Magisk", updateAvailable: "Pembaruan tersedia", moduleLabel: "Modul", appLabel: "Aplikasi", compatibleReleasePending: "Rilis yang kompatibel belum ditemukan",
    language: 'Bahasa', steamvrLifecycle: 'Mulai dan tutup bersama SteamVR', saveError: 'Pengaturan tidak dapat disimpan', openError: 'Tautan tidak dapat dibuka',
    styleGroup: "Gaya touchpad", styleAbsolute: "Posisi", styleJoystick: "Joystick", styleSwipe: "Usap", styleMouse: "Mouse", styleAbsoluteNote: "Titik sentuh apa adanya", styleJoystickNote: "Pergeseran dari titik sentuh", styleSwipeNote: "Kecepatan gerak dengan inersia", styleMouseNote: "Kursor Windows, klik tombol kiri", joystickRange: "Radius joystick", swipeGain: "Sensitivitas", mouseSpeed: "Kecepatan kursor", clickForce: "Tekan untuk klik", palmRejection: "Abaikan jempol yang bersandar", palmNeedsModule: "perlu modul v3.6+", calibrate: "Kalibrasi", calibrationTitle: "Kalibrasi touchpad", calibrationStep: "Langkah", calibrationEdges: "Telusuri tepi kedua panel dengan jari", calibrationUp: "Usap lurus ke atas di setiap panel", calibrationForce: "Sentuh ringan, lalu tekan sekuat tenaga", back: "Kembali", next: "Berikutnya", skip: "Lewati", save: "Simpan", reset: "Atur ulang", cancel: "Batal", steamvrDriver: "Driver SteamVR", driverClassic: "Klasik", driverClassicNote: "perangkat terpisah", driverFusionNote: "langsung di kontroler", driverOffline: "SteamVR tidak berjalan", driverRestart: "Mulai ulang SteamVR untuk menerapkan", driverFallback: "Fusion gagal dimulai; mode klasik berjalan", driverConflict: "QFTPlus terdeteksi: matikan touchpad-nya", driverClassicActive: "Mode klasik berjalan", driverFusionActive: "Fusion aktif", driverReconnect: "sambungkan ulang kontroler",
    quest3Title: "Apakah ini kontroler Quest Pro?", quest3Body: "SteamVR melihat kontroler Quest 3, yang bisa terjadi lewat Virtual Desktop. Jika ini Quest Pro, Fusion menambahkan touchpad dan menampilkan model Quest Pro setelah SteamVR dimulai ulang.", quest3Yes: "Ya, Quest Pro", quest3No: "Tidak", quest3Skipped: "Kontroler Quest 3 dilewati", change: "ubah",
  },
  ja: {
    tabSensors: 'センサー', tabMode: 'タッチパッド', tabStatus: '状態', tabSettings: '設定',
    stylusSensor: 'スタイラス', triggerProximitySensor: 'トリガー付近の指', triggerSlideSensor: 'トリガー表面のスライド',
    searching: '検索中', connecting: '接続中', connected: '接続済み', error: 'エラー',
    searchingHint: 'デバイスを検索中…', connectingHint: 'モジュールのデータを待機中…', connectedHint: 'センサーデータを受信中', errorHint: '接続エラー',
    left: '左', right: '右', online: '接続済み', offline: '未接続', battery: 'バッテリー',
    modeGroup: 'パッドのモード', touchpad: 'タッチパッド', position: '位置', button: 'ボタン', wholePad: '全体', twoButtons: '2ボタン', upperLower: '上 / 下',
    touchpadNote: '仮想ボタンを使わず位置と圧力を表示', pressStrength: '押す', releaseAt: '離す', hapticPower: '振動の強さ',
    force: '圧力', upper: '上', lower: '下', threshold: 'しきい値',
    bundledAdb: '内蔵 ADB', autoDiscovery: '自動検出', moduleRequired: 'Quest Pro で qptp-module を起動してください',
    updateRequired: "Quest Pro Touch Plus モジュールを更新", magiskUpdateHint: "Magisk のモジュールで「更新」を押すこともできます", updateAvailable: "更新があります", moduleLabel: "モジュール", appLabel: "アプリ", compatibleReleasePending: "互換性のあるリリースはまだ見つかりません",
    language: '言語', steamvrLifecycle: 'SteamVR と一緒に起動・終了', saveError: '設定を保存できませんでした', openError: 'リンクを開けませんでした',
    styleGroup: "タッチパッドのスタイル", styleAbsolute: "位置", styleJoystick: "ジョイスティック", styleSwipe: "スワイプ", styleMouse: "マウス", styleAbsoluteNote: "触れた位置をそのまま", styleJoystickNote: "触れた位置からのずれ", styleSwipeNote: "慣性つきの移動速度", styleMouseNote: "Windows のカーソル、クリックは左ボタン", joystickRange: "ジョイスティック半径", swipeGain: "感度", mouseSpeed: "カーソル速度", clickForce: "押してクリック", palmRejection: "置いた親指を無視", palmNeedsModule: "モジュール v3.6+ が必要", calibrate: "キャリブレーション", calibrationTitle: "タッチパッドのキャリブレーション", calibrationStep: "ステップ", calibrationEdges: "両方のパッドの縁を指でなぞってください", calibrationUp: "各パッドでまっすぐ上にスワイプ", calibrationForce: "軽く触れてから、思い切り押してください", back: "戻る", next: "次へ", skip: "スキップ", save: "保存", reset: "リセット", cancel: "キャンセル", steamvrDriver: "SteamVR ドライバー", driverClassic: "クラシック", driverClassicNote: "別デバイス", driverFusionNote: "コントローラーに統合", driverOffline: "SteamVR は起動していません", driverRestart: "反映するには SteamVR を再起動", driverFallback: "Fusion を開始できず、クラシックで動作中", driverConflict: "QFTPlus を検出：そのタッチパッドをオフに", driverClassicActive: "クラシックモードで動作中", driverFusionActive: "Fusion 有効", driverReconnect: "コントローラーを再接続してください",
    quest3Title: "これは Quest Pro コントローラーですか？", quest3Body: "SteamVR は Quest 3 コントローラーとして認識しています（Virtual Desktop 経由で起こることがあります）。Quest Pro であれば、SteamVR の再起動後に Fusion がタッチパッドを追加し、Quest Pro のモデルを表示します。", quest3Yes: "はい、Quest Pro", quest3No: "いいえ", quest3Skipped: "Quest 3 コントローラーをスキップ中", change: "変更",
  },
  de: {
    tabSensors: 'Sensoren', tabMode: 'Touchpads', tabStatus: 'Status', tabSettings: 'Einstellungen',
    stylusSensor: 'Stylus', triggerProximitySensor: 'Finger am Trigger', triggerSlideSensor: 'Wischen auf dem Trigger',
    searching: 'Suche läuft', connecting: 'verbinde', connected: 'verbunden', error: 'Fehler',
    searchingHint: 'Gerät wird gesucht…', connectingHint: 'Warte auf den Modulstream…', connectedHint: 'Sensorstream aktiv', errorHint: 'Verbindungsfehler',
    left: 'Links', right: 'Rechts', online: 'Verbunden', offline: 'Nicht verbunden', battery: 'Akku',
    modeGroup: 'Pad-Modus', touchpad: 'Touchpad', position: 'Position', button: 'Taste', wholePad: 'gesamtes Pad', twoButtons: '2 Tasten', upperLower: 'oben / unten',
    touchpadNote: 'Position und Druck ohne virtuelle Tasten', pressStrength: 'Drücken', releaseAt: 'Loslassen', hapticPower: 'Vibrationsstärke',
    force: 'DRUCK', upper: 'OBEN', lower: 'UNTEN', threshold: 'Schwelle',
    bundledAdb: 'integriertes ADB', autoDiscovery: 'automatische Suche', moduleRequired: 'qptp-module muss auf Quest Pro laufen',
    updateRequired: "Aktualisiere das Quest Pro Touch Plus Modul", magiskUpdateHint: "Du kannst auch in Magisk beim Modul auf Aktualisieren tippen", updateAvailable: "Update verfügbar", moduleLabel: "Modul", appLabel: "App", compatibleReleasePending: "Noch keine kompatible Version gefunden",
    language: 'Sprache', steamvrLifecycle: 'Mit SteamVR starten und beenden', saveError: 'Einstellung konnte nicht gespeichert werden', openError: 'Link konnte nicht geöffnet werden',
    styleGroup: "Touchpad-Stil", styleAbsolute: "Position", styleJoystick: "Joystick", styleSwipe: "Wischen", styleMouse: "Maus", styleAbsoluteNote: "Berührungspunkt unverändert", styleJoystickNote: "Abstand vom Berührungspunkt", styleSwipeNote: "Bewegungstempo mit Trägheit", styleMouseNote: "Windows-Cursor, Klick ist linke Taste", joystickRange: "Joystick-Radius", swipeGain: "Empfindlichkeit", mouseSpeed: "Cursortempo", clickForce: "Drücken zum Klicken", palmRejection: "Aufliegenden Daumen ignorieren", palmNeedsModule: "Modul v3.6+ nötig", calibrate: "Kalibrieren", calibrationTitle: "Touchpad-Kalibrierung", calibrationStep: "Schritt", calibrationEdges: "Fahre die Ränder beider Pads nach", calibrationUp: "Wische auf jedem Pad gerade nach oben", calibrationForce: "Leicht berühren, dann so fest wie möglich drücken", back: "Zurück", next: "Weiter", skip: "Überspringen", save: "Speichern", reset: "Zurücksetzen", cancel: "Abbrechen", steamvrDriver: "SteamVR-Treiber", driverClassic: "Klassisch", driverClassicNote: "separates Gerät", driverFusionNote: "direkt im Controller", driverOffline: "SteamVR läuft nicht", driverRestart: "SteamVR neu starten, um es zu übernehmen", driverFallback: "Fusion startete nicht; Klassisch läuft", driverConflict: "QFTPlus erkannt: schalte dessen Touchpad aus", driverClassicActive: "Klassischer Modus läuft", driverFusionActive: "Fusion aktiv", driverReconnect: "Controller neu verbinden",
    quest3Title: "Sind das Quest-Pro-Controller?", quest3Body: "SteamVR erkennt Quest-3-Controller, was über Virtual Desktop passieren kann. Sind es Quest Pro, fügt Fusion ihnen nach einem Neustart von SteamVR das Touchpad hinzu und zeigt das Quest-Pro-Modell.", quest3Yes: "Ja, Quest Pro", quest3No: "Nein", quest3Skipped: "Quest-3-Controller übersprungen", change: "ändern",
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
