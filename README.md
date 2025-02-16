# VULP (Validator Url:Login:Password)

Библиотека написанная на rust использующая SmallVector, memchr, url, vc

# Установка

Укажите библиотеку в `Cargo.toml`:
```toml
[dependencies]
vulp = { git = "https://github.com/BigBrainsClub/VULP" }
```

### **Пример использования**
```rust

use vulp::{LocalConfig, VULP};

fn main() -> std::io::Result<()> {
    let mut new_validator = VULP::new(&LocalConfig::default());
    let line = b"android://ojqioehtwpefnweuetwerwe@==example.com:gnweugbwekfnwe:ifaentqiewwer";
    assert!(match new_validator.validate(line)) {
        Ok(_) => true,
        e => {
            println!("{:?}", e)
        }
    }
    Ok(())
}
```

## Плюсы
1) Минимальное колличество аллокаций
2) Скорость

## todo
Написать с нуля валидацию url

## Лицензия
Эта библиотека распространяется под лицензией BSD 2-Clause. См. [LICENSE](LICENSE) для деталей.

## Авторы и вклад
 - [@BigBrainsClub](https://github.com/BigBrainsClub) - автор и разработчик
 - PR приветствуется! (Буду рад критике и улучшению данной библиотеки)