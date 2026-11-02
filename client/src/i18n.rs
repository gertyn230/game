//! Переклади інтерфейсу гри на 3 мови
use crate::settings::Language;

pub struct Texts {
    // Основне меню
    pub game_title: &'static str,
    pub loading: &'static str,
    pub connecting: &'static str,
    pub online: &'static str,

    // Дії
    pub action: &'static str,
    pub eat: &'static str,
    pub drink: &'static str,
    pub dig: &'static str,
    pub menu: &'static str,
    pub craft: &'static str,
    pub quick_craft: &'static str,
    pub close: &'static str,

    // Статистика
    pub hunger: &'static str,
    pub thirst: &'static str,
    pub energy: &'static str,
    pub health: &'static str,
    pub gold: &'static str,
    pub score: &'static str,

    // Меню
    pub inventory: &'static str,
    pub market: &'static str,
    pub leaderboard: &'static str,
    pub settings: &'static str,
    pub actions: &'static str,

    // Налаштування
    pub language: &'static str,
    pub music: &'static str,
    pub sounds: &'static str,
    pub volume: &'static str,
    pub keybindings: &'static str,
    pub controls: &'static str,

    // Повідомлення
    pub you_died: &'static str,
    pub items_dropped: &'static str,
    pub pick_up_items: &'static str,
    pub crafted: &'static str,
    pub not_enough: &'static str,

    // Інше
    pub yes: &'static str,
    pub no: &'static str,
    pub ok: &'static str,
    pub cancel: &'static str,
}

const UA: Texts = Texts {
    game_title: "Pixel Village",
    loading: "Завантаження...",
    connecting: "Підключення до сервера...",
    online: "Онлайн",

    action: "Добути",
    eat: "Їсти",
    drink: "Пити",
    dig: "Копати",
    menu: "Меню",
    craft: "Крафт",
    quick_craft: "Швидкий крафт",
    close: "Закрити",

    hunger: "Голод",
    thirst: "Спрага",
    energy: "Енергія",
    health: "Здоров'я",
    gold: "Золото",
    score: "Очки",

    inventory: "Інвентар",
    market: "Ринок",
    leaderboard: "Рейтинг",
    settings: "Налаштування",
    actions: "Дії",

    language: "Мова",
    music: "Музика",
    sounds: "Звуки",
    volume: "Гучність",
    keybindings: "Клавіші",
    controls: "Керування",

    you_died: "Ви померли",
    items_dropped: "Речі залишилися на місці смерті",
    pick_up_items: "💀 Ваші речі",
    crafted: "Створено",
    not_enough: "Недостатньо ресурсів",

    yes: "Так",
    no: "Ні",
    ok: "Гаразд",
    cancel: "Скасувати",
};

const EN: Texts = Texts {
    game_title: "Pixel Village",
    loading: "Loading...",
    connecting: "Connecting to server...",
    online: "Online",

    action: "Gather",
    eat: "Eat",
    drink: "Drink",
    dig: "Dig",
    menu: "Menu",
    craft: "Craft",
    quick_craft: "Quick Craft",
    close: "Close",

    hunger: "Hunger",
    thirst: "Thirst",
    energy: "Energy",
    health: "Health",
    gold: "Gold",
    score: "Score",

    inventory: "Inventory",
    market: "Market",
    leaderboard: "Leaderboard",
    settings: "Settings",
    actions: "Actions",

    language: "Language",
    music: "Music",
    sounds: "Sounds",
    volume: "Volume",
    keybindings: "Keybindings",
    controls: "Controls",

    you_died: "You died",
    items_dropped: "Items remained at death location",
    pick_up_items: "💀 Your items",
    crafted: "Crafted",
    not_enough: "Not enough resources",

    yes: "Yes",
    no: "No",
    ok: "OK",
    cancel: "Cancel",
};

const RU: Texts = Texts {
    game_title: "Pixel Village",
    loading: "Загрузка...",
    connecting: "Подключение к серверу...",
    online: "Онлайн",

    action: "Добыть",
    eat: "Есть",
    drink: "Пить",
    dig: "Копать",
    menu: "Меню",
    craft: "Крафт",
    quick_craft: "Быстрый крафт",
    close: "Закрыть",

    hunger: "Голод",
    thirst: "Жажда",
    energy: "Энергия",
    health: "Здоровье",
    gold: "Золото",
    score: "Очки",

    inventory: "Инвентарь",
    market: "Рынок",
    leaderboard: "Рейтинг",
    settings: "Настройки",
    actions: "Действия",

    language: "Язык",
    music: "Музыка",
    sounds: "Звуки",
    volume: "Громкость",
    keybindings: "Клавиши",
    controls: "Управление",

    you_died: "Вы погибли",
    items_dropped: "Вещи остались на месте смерти",
    pick_up_items: "💀 Ваши вещи",
    crafted: "Создано",
    not_enough: "Недостаточно ресурсов",

    yes: "Да",
    no: "Нет",
    ok: "OK",
    cancel: "Отмена",
};

pub fn get_texts(lang: &Language) -> &'static Texts {
    match lang {
        Language::Ukrainian => &UA,
        Language::English => &EN,
        Language::Russian => &RU,
    }
}
