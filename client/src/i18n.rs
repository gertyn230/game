//! Переклади інтерфейсу гри на 3 мови
use crate::settings::Language;

pub struct Texts {
    // Основне меню
    pub game_title: &'static str,
    pub loading: &'static str,
    pub loading_assets: &'static str,
    pub loading_world: &'static str,
    pub connecting: &'static str,
    pub online: &'static str,
    pub play: &'static str,
    pub continue_game: &'static str,
    pub new_game: &'static str,
    pub quit: &'static str,

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
    pub music_volume: &'static str,
    pub sound_volume: &'static str,
    pub apply: &'static str,

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

    // Предмети (Items)
    pub item_names: [&'static str; 19],

    // Інструменти (Tools)
    pub tool_names: [&'static str; 14],

    // Рецепти
    pub recipe_names: [&'static str; 23],
}

const UA: Texts = Texts {
    game_title: "Pixel Village",
    loading: "Завантаження...",
    loading_assets: "Завантаження ресурсів...",
    loading_world: "Генерація світу...",
    connecting: "Підключення до сервера...",
    online: "Онлайн",
    play: "Грати",
    continue_game: "Продовжити",
    new_game: "Нова гра",
    quit: "Вийти",

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
    music_volume: "Гучність музики",
    sound_volume: "Гучність звуків",
    apply: "Застосувати",

    you_died: "Ви померли",
    items_dropped: "Речі залишилися на місці смерті",
    pick_up_items: "💀 Ваші речі",
    crafted: "Створено",
    not_enough: "Недостатньо ресурсів",

    yes: "Так",
    no: "Ні",
    ok: "Гаразд",
    cancel: "Скасувати",

    item_names: [
        "Дерево", "Камінь", "Їжа", "Палиця", "Камінчик", "Нитка", "Дошки", "Пайок",
        "М'ясо", "Смажене", "Шкіра", "Черв'як", "Риба", "Смажена риба", "Вугілля",
        "Мідна руда", "Залізна руда", "Мідь", "Залізо",
    ],

    tool_names: [
        "Кам'яна сокира", "Кам'яна кирка", "Мідна сокира", "Мідна кирка",
        "Залізна сокира", "Залізна кирка", "Списа", "Ніж", "Лопата", "Вудка",
        "Кошик", "Човен", "Хутряний одяг", "Кольчуга",
    ],

    recipe_names: [
        "Вогнище", "Списа", "Смажене м'ясо", "Смажена риба", "Пайок",
        "Кам'яний ніж", "Кам'яна сокира", "Кам'яна кирка", "Лопата", "Вудка",
        "Кошик", "Дошки", "Піч", "Виплавити мідь", "Виплавити залізо",
        "Мідна сокира", "Мідна кирка", "Залізна сокира", "Залізна кирка",
        "Хутряний одяг", "Кольчуга", "Човен", "Будинок",
    ],
};

const EN: Texts = Texts {
    game_title: "Pixel Village",
    loading: "Loading...",
    loading_assets: "Loading assets...",
    loading_world: "Generating world...",
    connecting: "Connecting to server...",
    online: "Online",
    play: "Play",
    continue_game: "Continue",
    new_game: "New Game",
    quit: "Quit",

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
    music_volume: "Music Volume",
    sound_volume: "Sound Volume",
    apply: "Apply",

    you_died: "You died",
    items_dropped: "Items remained at death location",
    pick_up_items: "💀 Your items",
    crafted: "Crafted",
    not_enough: "Not enough resources",

    yes: "Yes",
    no: "No",
    ok: "OK",
    cancel: "Cancel",

    item_names: [
        "Wood", "Stone", "Food", "Stick", "Pebble", "Fiber", "Plank", "Ration",
        "Meat", "Roast", "Hide", "Worm", "Fish", "Roasted Fish", "Coal",
        "Copper Ore", "Iron Ore", "Copper", "Iron",
    ],

    tool_names: [
        "Stone Axe", "Stone Pickaxe", "Copper Axe", "Copper Pickaxe",
        "Iron Axe", "Iron Pickaxe", "Spear", "Knife", "Shovel", "Fishing Rod",
        "Basket", "Boat", "Fur Coat", "Chainmail",
    ],

    recipe_names: [
        "Campfire", "Spear", "Roasted Meat", "Roasted Fish", "Ration",
        "Stone Knife", "Stone Axe", "Stone Pickaxe", "Shovel", "Fishing Rod",
        "Basket", "Planks", "Furnace", "Smelt Copper", "Smelt Iron",
        "Copper Axe", "Copper Pickaxe", "Iron Axe", "Iron Pickaxe",
        "Fur Coat", "Chainmail", "Boat", "House",
    ],
};

const RU: Texts = Texts {
    game_title: "Pixel Village",
    loading: "Загрузка...",
    loading_assets: "Загрузка ресурсов...",
    loading_world: "Генерация мира...",
    connecting: "Подключение к серверу...",
    online: "Онлайн",
    play: "Играть",
    continue_game: "Продолжить",
    new_game: "Новая игра",
    quit: "Выйти",

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
    music_volume: "Громкость музыки",
    sound_volume: "Громкость звуков",
    apply: "Применить",

    you_died: "Вы погибли",
    items_dropped: "Вещи остались на месте смерти",
    pick_up_items: "💀 Ваши вещи",
    crafted: "Создано",
    not_enough: "Недостаточно ресурсов",

    yes: "Да",
    no: "Нет",
    ok: "OK",
    cancel: "Отмена",

    item_names: [
        "Дерево", "Камень", "Еда", "Палка", "Камешек", "Нить", "Доски", "Паёк",
        "Мясо", "Жаркое", "Шкура", "Червь", "Рыба", "Жареная рыба", "Уголь",
        "Медная руда", "Железная руда", "Медь", "Железо",
    ],

    tool_names: [
        "Каменный топор", "Каменная кирка", "Медный топор", "Медная кирка",
        "Железный топор", "Железная кирка", "Копьё", "Нож", "Лопата", "Удочка",
        "Корзина", "Лодка", "Меховая одежда", "Кольчуга",
    ],

    recipe_names: [
        "Костёр", "Копьё", "Жаркое из мяса", "Жареная рыба", "Паёк",
        "Каменный нож", "Каменный топор", "Каменная кирка", "Лопата", "Удочка",
        "Корзина", "Доски", "Печь", "Выплавить медь", "Выплавить железо",
        "Медный топор", "Медная кирка", "Железный топор", "Железная кирка",
        "Меховая одежда", "Кольчуга", "Лодка", "Дом",
    ],
};

pub fn get_texts(lang: &Language) -> &'static Texts {
    match lang {
        Language::Ukrainian => &UA,
        Language::English => &EN,
        Language::Russian => &RU,
    }
}
