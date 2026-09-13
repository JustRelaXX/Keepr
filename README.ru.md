# Keepr 🌱

[![Desktop checks](https://github.com/JustRelaXX/Keepr/actions/workflows/check.yml/badge.svg)](https://github.com/JustRelaXX/Keepr/actions/workflows/check.yml)
[![Latest release](https://img.shields.io/github/v/release/JustRelaXX/Keepr)](https://github.com/JustRelaXX/Keepr/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

**Маленькая забота. Большой уют.** Keepr — локальный desktop-хранитель дома:
напоминает о замене, обслуживании и сроках годности вещей. Никакой
регистрации и облака — всё хранится в SQLite на устройстве.

English version: [README.md](README.md).

## Установка

Скачай последний выпуск со страницы
[**Releases**](https://github.com/JustRelaXX/Keepr/releases/latest):

| ОС | Файл | Как установить |
|----|------|----------------|
| Fedora / RHEL | `Keepr_*_x86_64.rpm` | `sudo dnf install ./Keepr_*.rpm` |
| Ubuntu / Debian | `Keepr_*_amd64.deb` | `sudo apt install ./Keepr_*.deb` |
| Arch Linux | AUR `keepr-bin` | `yay -S keepr-bin` или `paru -S keepr-bin` |
| Любой Linux | `Keepr_*.AppImage` | `chmod +x Keepr_*.AppImage && ./Keepr_*.AppImage` |
| Windows 10/11 | `Keepr_*_x64-setup.exe` | Запустить установщик (WebView2 обычно уже есть) |

После установки включи в настройках *«Встречать тебя при входе в систему»*,
чтобы работали фоновые напоминания. Полный выход — **трей → «Выйти»**:
закрытие окна напоминания не останавливает.

> В GNOME без расширения AppIndicator иконки сторонних приложений в трее могут
> не отображаться. Keepr остаётся доступен через launcher.

![Дом](docs/screenshots/home-light.png)
![Все вещи](docs/screenshots/things-light.png)
![Карточка вещи](docs/screenshots/thing-details.png)
![Календарь](docs/screenshots/calendar-light.png)
![Тёмная тема](docs/screenshots/dashboard-dark.png)

## Что работает

- Разовые и повторяющиеся вещи: дни, недели, календарные месяцы и годы.
- Быстрое добавление из 30 конкретных шаблонов или с нуля.
- Комнаты с собственным оформлением, поиск, фильтры, сортировка.
- Выполнение сегодня или задним числом; отмена выполнения и удаления.
- «Напомнить позже»: через час, до вечера, завтра. Срок вещи сохраняется.
- Календарь текущих сроков, общий журнал и история каждой вещи.
- Удаление отдельных записей журнала с отменой через «Вернуть».
- Здоровье дома, три SVG-хранителя, уровни заботы.
- Светлая/тёмная/системная темы; русский и английский языки.
- Нативные уведомления **без кнопок**, тихие часы, квитанции доставки.
- Tray с быстрыми действиями, автозапуск при входе в систему.
- Фоновый Rust-процесс без открытого WebView; окно создаётся по запросу.
- Резервные копии `.keepr` и восстановление с автоматической копией.

## Приватность

- Полностью офлайн и local-first. Вещи, комнаты, история и настройки — в SQLite
  на устройстве (`~/.local/share/app.keepr.desktop/keepr.db` в Linux,
  `%APPDATA%` в Windows).
- Нет телеметрии, аккаунтов и сетевых запросов.

## Разработка

Нужны Node.js 22.12+ и стабильный Rust.

```sh
# Fedora
sudo dnf install gtk3-devel webkit2gtk4.1-devel libappindicator-gtk3-devel \
  librsvg2-devel libxdo-devel openssl-devel gcc-c++ make
# Ubuntu / Debian
sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \
  librsvg2-dev libxdo-dev libssl-dev build-essential
# Arch
sudo pacman -S --needed webkit2gtk-4.1 base-devel curl wget file openssl \
  appmenu-gtk-module libappindicator-gtk3 librsvg xdotool
# Windows: Rust MSVC + VS Build Tools (Desktop development with C++) +
# Node.js + WebView2 Runtime
npm ci
npm run tauri dev
```

`npm run dev` запускает только frontend. Используй **`npm run tauri dev`**:
persistence и бизнес-логика находятся в Rust.

### Проверки

```sh
cargo test -p keepr-core
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
npm run check
npm test
npm run tauri build -- --debug --no-bundle
npm run test:e2e
```

Desktop E2E использует настоящий Tauri/WebKitGTK, отдельный временный профиль
и настоящую SQLite. Дополнительно:

```sh
KEEPR_TEST_NOTIFICATIONS=1 npm run test:e2e  # реальная доставка уведомлений
node tests/desktop.mjs --demo                # скриншоты демо-дома
KEEPR_BINARY=target/release/keepr npm run test:e2e  # проверка релизного бинаря
```

Подробности: [архитектура](docs/architecture.md),
[статус платформ](docs/platforms.md).

## Структура проекта

- `src/` — Svelte 5 + TypeScript, только presentation.
- `crates/keepr-core/` — доменная модель, расписания, SQLite, миграции.
- `src-tauri/` — IPC-команды, tray, уведомления, автозапуск, фон.
- `resources/` — общие RU/EN-строки, пресеты, артворк.
- `docs/` — архитектура, платформы, QA, release notes.
- `packaging/aur/` — исходники AUR-пакета (`keepr-bin`).

## Лицензия

MIT — см. [LICENSE](LICENSE). Иллюстрации — локальные SVG; Manrope — SIL OFL,
иконки Lucide — ISC.
