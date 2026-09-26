# QPTP Auxiliary OpenVR driver

Один отдельный controller-class device с ролью `TrackedControllerRole_Treadmill`. Он не заменяет и не изменяет Quest Pro controllers. Все 12 компонентов (два pad по X/Y/touch/force и четыре кнопки) создаются в `Activate()` и остаются в профиле при любом режиме. Приложение QPTP передаёт реальные показания модуля в драйвер через локальный UDP. Источник ввода отделён интерфейсом `IInputSource`; внешний тестовый отправитель также доступен.

## Сборка

Нужны Windows x64 и Visual Studio 2022 с C++ toolchain и CMake. В корне проекта:

```powershell
cmake -S .\openvr-aux -B .\openvr-aux\build -G "Visual Studio 17 2022" -A x64
cmake --build .\openvr-aux\build --config Release
```

Если CMake не установлен, используйте `powershell -File .\openvr-aux\build-driver.ps1` с Visual Studio 2022 Build Tools.

DLL окажется в `openvr-aux\driver_qptp\bin\win64`. OpenVR driver header взят из официального SDK, копия и его лицензия находятся в `vendor/openvr`. Для DLL не нужен `openvr_api.dll`: driver API предоставляет SteamVR через `IVRDriverContext`.

## Установка

Закройте SteamVR. Укажите свой путь к SteamVR при необходимости:

```powershell
$vr = "C:\Program Files (x86)\Steam\steamapps\common\SteamVR"
$driver = (Resolve-Path .\openvr-aux\driver_qptp).Path
& "$vr\bin\win64\vrpathreg.exe" adddriver "$driver"
& "$vr\bin\win64\vrpathreg.exe" show
```

Запустите SteamVR с уже работающими Quest Pro controllers. Не устанавливайте одновременно прежний экспериментальный `qproxy` driver, если он был отдельно зарегистрирован: удалите его через `vrpathreg.exe removedriver <полный путь к старому driver_qproxy>`.

Установщик QPTP включает этот пакет и автоматически регистрирует его при установке. `driver.vrdrivermanifest` задаёт `alwaysActivate=true`, чтобы SteamVR загрузил драйвер вместе с другим HMD. Формат этого манифеста не имеет поля для запуска обычного Windows-приложения; после загрузки установленный драйвер запускает соседний `qptp.exe`. Повторные экземпляры приложения завершаются через single-instance mutex. Если SteamVR отсутствовал во время установки, запуск установленного приложения повторит регистрацию после появления SteamVR.

## Проверка

1. В логе SteamVR `logs\vrserver.txt` найдите `[qptp] Driver initialized`, `Device registered`, `Device activated index=...`, `Input component created` и `Debug UDP listening`.
2. В SteamVR откройте **Settings → Controllers → Manage Controller Bindings**, выберите приложение и откройте дополнительные bindings для устройства QPTP Auxiliary Input. Отдельный источник `/user/treadmill` содержит Left/Right Pad и четыре кнопки. Эта схема проверена в текущей установке SteamVR.
3. Запустите приложение из проекта через `dev.bat` или `npm run tauri dev`. Подключите Quest-модуль по USB или LAN и выберите режим в приложении. В режиме **Touchpad** приложение передаёт X/Y/touch/force. В режимах **1 button** и **2 buttons** оно передаёт кнопки и обнуляет тачпады. При пропаже соединения всё обнуляется.
4. В `vrserver.txt` появятся строки `Input update` с живыми значениями. Это проверяет путь от сенсоров приложения до `IVRDriverInput`. Для отдельной проверки без приложения отправьте `powershell -File .\openvr-aux\send-debug.ps1 -Mode touchpad -Side left -Touch -X 0.4 -Y -0.2 -Force 0.8`. Значения сбрасываются через 2 секунды без новых UDP пакетов.
5. Назначьте, например, `left_pad` на movement и `right_extra_1/click` на действие игры. Проверьте, что штатные Quest Pro bindings продолжают отвечать.

Профиль находится в `resources/input/qptp_profile.json`. Для него указан обязательный `/pose/raw`. Устройство сообщает `deviceIsConnected=true`, `poseIsValid=false` и identity quaternion: реального tracking нет. Нужно проверить в запущенном SteamVR, принимает ли ваша версия input в таком состоянии. Если нет, следующий шаг — стационарный валидный pose без привязки к Quest. `Prop_RenderModelName_String` задан для идентификации, но 3D render model в MVP отсутствует; Binding UI использует отдельный SVG.

UDP слушает только `127.0.0.1:39571`; формат — строки `key=value`, допускаются `\n` или запятые. Ключи: `left_pad_x`, `left_pad_y`, `left_pad_touch`, `left_pad_force`, аналоги `right_*`, а также `left_extra_1`, `left_extra_2`, `right_extra_1`, `right_extra_2`. X/Y ограничены [-1,1], force [0,1], bool — 0 или любое ненулевое число.

## Удаление

Закройте SteamVR и выполните:

```powershell
$vr = "C:\Program Files (x86)\Steam\steamapps\common\SteamVR"
$driver = (Resolve-Path .\openvr-aux\driver_qptp).Path
& "$vr\bin\win64\vrpathreg.exe" removedriver "$driver"
```

После этого можно удалить папку драйвера. Логирование идёт через `IVRDriverLog` в `vrserver.txt`, включая коды ошибок свойств и input API. Частые обновления логируются только при получении пакета или сбросе; компоненты обновляются каждый кадр.

## Проверка в текущем окружении

DLL скомпилирована MSVC x64, приложение прошло `cargo test` и `npm run build`. В запущенном SteamVR зарегистрированы два штатных Quest Pro controller и один `qptp_input`. Лог подтвердил живые X/Y/touch/force из приложения; пользователь подтвердил работу в SteamVR. При использовании уже занятой роли Treadmill SteamVR выбирает лишь одно активное устройство этой роли.
