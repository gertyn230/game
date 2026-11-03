//! Автоматичний переклад ігрових повідомлень
use crate::i18n::Texts;

pub fn translate_message<'a>(msg: &str, texts: &'a Texts) -> &'a str {
    match msg {
        // Тварини
        "Кролика поймает только копьё" => texts.need_spear,
        "Нужен нож: палка, камешек, нить" => texts.need_knife,
        "Туша разделана: мясо и шкура" => texts.corpse_butchered,
        "Разделай тушу: нужен каменный нож" => texts.butcher_corpse,

        // Їжа
        "Сырое мясо вредно — пожарь у костра" => texts.raw_meat_bad,
        "Сырая рыба — лучше пожарь" => texts.raw_fish_bad,

        // Вода
        "Ты пьёшь воду" => texts.drinking_water,
        "Воды рядом нет" => texts.no_water_nearby,

        // Копання
        "Нужна лопата: палки, камешки, нить" => texts.need_shovel,
        "Тут нельзя копать" => texts.cant_dig_here,
        "Плато слишком твёрдое" => texts.plateau_too_hard,
        "Здесь уже перекопано" => texts.already_dug,
        "Нашёл червя" => texts.found_worm,
        "Пусто, попробуй в другом месте" => texts.nothing_found,

        // Міст
        "Из лодки мост не построить" => texts.cant_build_from_boat,
        "Встань лицом к воде" => texts.face_water,
        "Нужно 2 доски" => texts.need_planks,
        "Мост построен" => texts.bridge_built,

        // Човен
        "Нужна лодка: 10 досок и 4 нити" => texts.need_boat,
        "Нужен свободный берег" => texts.need_free_shore,
        "Подойди к воде" => texts.approach_water,
        "Ты в лодке" => texts.in_boat,
        "Ты на берегу" => texts.on_shore,

        // Вовк/питомець
        "Рядом нет волка" => texts.no_wolf_nearby,
        "У тебя уже есть питомец" => texts.already_have_pet,
        "Нужно сырое мясо" => texts.need_raw_meat,
        "Волк стал твоим другом!" => texts.wolf_is_friend,
        "Волк кусает!" => texts.wolf_bites,

        // Смерть/речі
        "Забрал свої речі!" => texts.picked_up_items,
        "Ти помер. Поверніся на місце смерті, щоб забрати речі!" => texts.you_died_message,

        // Рибалка
        "Закинул удочку, жди поклёвку" => texts.cast_rod,
        "Ты отошёл — удочка смотана" => texts.moved_rod_retracted,
        "Ты смотал удочку" => texts.retracted_rod,
        "Клюёт! Жми «Добыть»!" => texts.fish_biting,
        "Тяни! Держи полоску в зелёной зоне" => texts.pull_keep_green,
        "Рыба сорвалась" => texts.fish_escaped,
        "Леска порвалась!" => texts.line_broke,
        "Рыба ушла" => texts.fish_got_away,

        // Кирка/видобування
        "Нужна кирка: палка, камешки, нить" |
        "Нужна кирка: палки, камешки, нить" => texts.need_pickaxe,

        // Якщо повідомлення не знайдено - повертаємо оригінал
        _ => msg,
    }
}
