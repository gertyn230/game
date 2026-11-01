//! Данные игры: предметы, инструменты, рецепты (отдельно от логики, чтобы легко добавлять).

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum It {
    Wood, Stone, Food, Stick, Pebble, Fiber, Plank, Meal, Meat, Roast, Hide, Worm, Fish, RFish, Coal, CuOre, FeOre, Cu, Fe,
}
pub const NI: usize = 19;
pub const IT_NAMES: [&str; NI] = [
    "Дерево", "Камень", "Еда", "Палка", "Камешек", "Нить", "Доски", "Паёк", "Мясо", "Жаркое", "Шкура", "Червь", "Рыба",
    "Жареная рыба", "Уголь", "Медная руда", "Железная руда", "Медь", "Железо",
];
pub const IT_COLOR: [u32; NI] = [
    0x8b5a2b, 0x8d8d96, 0xd94f4f, 0xb07a45, 0x9a9aa4, 0x9ad06a, 0xc9a06a, 0xe0b070, 0xb03a3a, 0xc26a2a, 0x7a5a3a, 0xd98a8a,
    0x6aa0d0, 0xe0a050, 0x222228, 0xd9823a, 0xb5613a, 0xe08a3a, 0xc8ccd4,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum Tl {
    Axe, Pick, Axe2, Pick2, Axe3, Pick3, Spear, Knife, Shovel, Rod, Basket, Boat, Coat, Mail,
}
pub const NT: usize = 14;
pub const TL_NAMES: [&str; NT] = [
    "Каменный топор", "Каменная кирка", "Медный топор", "Медная кирка", "Железный топор", "Железная кирка", "Копьё", "Нож",
    "Лопата", "Удочка", "Корзина", "Лодка", "Меховая одежда", "Кольчуга",
];
pub const TL_DUR: [i32; NT] = [40, 40, 80, 80, 160, 160, 20, 15, 30, 25, 25, 1, 1, 1];

pub enum Out {
    Item(It, u32),
    Tool(Tl),
    Fire,
    Furnace,
    House,
}

pub struct Recipe {
    pub name: &'static str,
    pub cost: &'static [(It, u32)],
    pub out: Out,
    pub fire: bool,
    pub furnace: bool,
}

use It::*;
pub const RECIPES: &[Recipe] = &[
    Recipe { name: "Костёр", cost: &[(Wood, 2), (Stick, 3)], out: Out::Fire, fire: false, furnace: false },
    Recipe { name: "Копьё", cost: &[(Stick, 2), (Pebble, 1), (Fiber, 1)], out: Out::Tool(Tl::Spear), fire: false, furnace: false },
    Recipe { name: "Жаркое из мяса", cost: &[(Meat, 1)], out: Out::Item(Roast, 1), fire: true, furnace: false },
    Recipe { name: "Жареная рыба", cost: &[(Fish, 1)], out: Out::Item(RFish, 1), fire: true, furnace: false },
    Recipe { name: "Паёк", cost: &[(Food, 2)], out: Out::Item(Meal, 1), fire: false, furnace: false },
    Recipe { name: "Каменный нож", cost: &[(Stick, 1), (Pebble, 1), (Fiber, 1)], out: Out::Tool(Tl::Knife), fire: false, furnace: false },
    Recipe { name: "Каменный топор", cost: &[(Stick, 1), (Pebble, 2), (Fiber, 1)], out: Out::Tool(Tl::Axe), fire: false, furnace: false },
    Recipe { name: "Каменная кирка", cost: &[(Stick, 1), (Pebble, 3), (Fiber, 1)], out: Out::Tool(Tl::Pick), fire: false, furnace: false },
    Recipe { name: "Лопата", cost: &[(Stick, 2), (Pebble, 2), (Fiber, 1)], out: Out::Tool(Tl::Shovel), fire: false, furnace: false },
    Recipe { name: "Удочка", cost: &[(Stick, 2), (Fiber, 3)], out: Out::Tool(Tl::Rod), fire: false, furnace: false },
    Recipe { name: "Корзина", cost: &[(Stick, 3), (Fiber, 4)], out: Out::Tool(Tl::Basket), fire: false, furnace: false },
    Recipe { name: "Доски", cost: &[(Wood, 3)], out: Out::Item(Plank, 2), fire: false, furnace: false },
    Recipe { name: "Печь", cost: &[(Stone, 10)], out: Out::Furnace, fire: false, furnace: false },
    Recipe { name: "Выплавить медь", cost: &[(CuOre, 2), (Coal, 1)], out: Out::Item(Cu, 1), fire: false, furnace: true },
    Recipe { name: "Выплавить железо", cost: &[(FeOre, 2), (Coal, 1)], out: Out::Item(Fe, 1), fire: false, furnace: true },
    Recipe { name: "Медный топор", cost: &[(Stick, 1), (Cu, 3), (Fiber, 1)], out: Out::Tool(Tl::Axe2), fire: false, furnace: false },
    Recipe { name: "Медная кирка", cost: &[(Stick, 1), (Cu, 4), (Fiber, 1)], out: Out::Tool(Tl::Pick2), fire: false, furnace: false },
    Recipe { name: "Железный топор", cost: &[(Stick, 1), (Fe, 3), (Fiber, 1)], out: Out::Tool(Tl::Axe3), fire: false, furnace: false },
    Recipe { name: "Железная кирка", cost: &[(Stick, 1), (Fe, 4), (Fiber, 1)], out: Out::Tool(Tl::Pick3), fire: false, furnace: false },
    Recipe { name: "Меховая одежда", cost: &[(Hide, 3), (Fiber, 2)], out: Out::Tool(Tl::Coat), fire: false, furnace: false },
    Recipe { name: "Кольчуга", cost: &[(Fe, 6), (Fiber, 2)], out: Out::Tool(Tl::Mail), fire: false, furnace: false },
    Recipe { name: "Лодка", cost: &[(Plank, 10), (Fiber, 4)], out: Out::Tool(Tl::Boat), fire: false, furnace: false },
    Recipe { name: "Дом", cost: &[(Plank, 8), (Stone, 10)], out: Out::House, fire: false, furnace: false },
];
