# Object Safety в Rust: почему не любой трейт может быть `dyn Trait`

## Что такое object safety

**Object safety** (безопасность для объектов) — это **набор правил**, которым должен соответствовать трейт, чтобы его можно было использовать как **`dyn Trait`**. Если трейт **не** object-safe, компилятор **запретит** `dyn Trait` и выдаст ошибку.

**Причина:** для `dyn Trait` нужна **vtable** — таблица с **фиксированными** слотами и **известными** сигнатурами. Если сигнатура зависит от **конкретного** типа (`Self`), vtable **невозможна**.

## Разбор примера

### Трейт, который **НЕ** object-safe

```rust
trait MyNotDynTrait {
    fn make(&self) -> Self;   // ← возвращает Self
}
```

**Почему `!dyn`-совместим:**

- **`make(&self) -> Self`** возвращает `Self` **по значению**.
- **Размер** `Self` **зависит** от конкретного типа (`S1` ≠ `S2`).
- **Vtable** должна знать **размер** возвращаемого значения.
- Компилятор **не может** создать vtable → **`!dyn`**.

### Попытка использовать как `dyn`

```rust
fn use_dyn(v: &dyn MyNotDynTrait) { ... }   // ❌ ошибка
```

**Ошибка:**

```
error[E0038]: the trait `MyNotDynTrait` cannot be made into an object
  --> src/main.rs:5:13
   |
5  | fn use_dyn(v: &dyn MyNotDynTrait) {
   |             ^^^^^^^^^^^^^^^^^^^^ `MyNotDynTrait` cannot be made into an object
   |
   = note: method `make` references the `Self` type in its return type
```

### Трейт, который **object-safe**

```rust
trait MyDynTrait {
    fn name(&self) -> &str;   // ← возвращает &str, не Self
}
```

**Почему `dyn`-совместим:**

- **`name(&self) -> &str`** — возвращает **`&str`** (фиксированный тип).
- **Размер** известен **заранее**.
- **Vtable** может быть построена.
- **`&dyn MyDynTrait`** — **работает**.

### Использование

```rust
fn use_dyn(v: &dyn MyDynTrait) {
    println!("{}", v.name());
}

let v_dyn_tr = S2 { v: "Dynamic Trait".to_string() };
use_dyn(&v_dyn_tr);
```

## Правила object safety

### 1. Методы **не** принимают и **не** возвращают `Self` **по значению**

```rust
// ❌ НЕ object-safe
fn make(&self) -> Self;
fn consume(self);
fn clone_self(self) -> Self;

// ✅ Object-safe
fn name(&self) -> &str;
fn make_ref(&self) -> &Self;
fn make_box(&self) -> Box<Self>;   // ✅ (Box<Self> — исключение)
```

**Исключение:** `Box<Self>`, `&Self`, `&mut Self` — **разрешены** (это указатели фиксированного размера).

### 2. `Self` **не** появляется в **ассоциированных константах**

```rust
// ❌ НЕ object-safe
trait Bad {
    const SIZE: usize = std::mem::size_of::<Self>();
}
```

**Почему:** `SIZE` **зависит** от конкретного типа.

### 3. **Нет** generic-методов

```rust
// ❌ НЕ object-safe
trait Bad {
    fn process<T>(&self, x: T);
}
```

**Почему:** `T` **неизвестен** на этапе vtable. Для **каждого** `T` — **свой** метод → **бесконечная** vtable.

```rust
// ✅ Object-safe
trait Good {
    fn process(&self, x: i32);   // ← фиксированный тип
}
```

### 4. **Нет** `where Self: Sized` **на всём трейте**

```rust
// ❌ НЕ object-safe
trait Bad where Self: Sized {
    fn name(&self) -> &str;
}
```

**Почему:** `where Self: Sized` **требует** известного размера → **несовместимо** с `dyn`.

**Исключение:** `where Self: Sized` **на отдельных методах**:

```rust
// ✅ Object-safe
trait Good {
    fn name(&self) -> &str;

    fn clone_self(&self) -> Self
    where
        Self: Sized;   // ← метод не в vtable
}
```

**Метод** с `where Self: Sized` **не попадает** в vtable, но **сам трейт** остаётся object-safe.

## Сводная таблица

| Правило | ❌ НЕ object-safe | ✅ Object-safe |
|---|---|---|
| Возврат `Self` | `fn make(&self) -> Self` | `fn make(&self) -> Box<Self>` |
| Приём `self` | `fn consume(self)` | `fn name(&self)` |
| `Self` в константе | `const SIZE: usize` | — |
| Generic-метод | `fn process<T>(&self, x: T)` | `fn process(&self, x: i32)` |
| `where Self: Sized` на всём трейте | `trait T where Self: Sized` | `trait T` |
| `where Self: Sized` на методе | — | ✅ Разрешено |

## Почему это **важно**

### Vtable — фиксированный набор слотов

```
Vtable для dyn MyDynTrait:
+------------------+
| drop_in_place    |
+------------------+
| size             |
+------------------+
| align            |
+------------------+
| name() -> &str   |   ← фиксированная сигнатура
+------------------+
```

**Каждый слот** имеет **фиксированный** размер и сигнатуру. Если бы метод возвращал `Self`, размер **зависел бы** от конкретного типа → vtable **невозможна**.

### `Box<Self>` — исключение

```rust
fn make(&self) -> Box<Self>;   // ✅ Object-safe
```

**Почему:** `Box<Self>` — **указатель** фиксированного размера (8 байт). Размер **самого** `Self` — **внутри** кучи, за указателем. Vtable может вернуть `Box<Self>` → **работает**.

## Пример: object-safe с `Box<Self>`

```rust
trait Cloneable {
    fn clone_box(&self) -> Box<dyn Cloneable>;
}

struct S;

impl Cloneable for S {
    fn clone_box(&self) -> Box<dyn Cloneable> {
        Box::new(S)
    }
}
```

**`clone_box`** возвращает `Box<dyn Cloneable>` — **фиксированный** размер → **object-safe**.

## Стандартный пример: `Clone` — **не** object-safe

```rust
pub trait Clone {
    fn clone(&self) -> Self;   // ← возвращает Self
}
```

**`Clone: !dyn`-совместим** — нельзя `dyn Clone`. Именно поэтому в стандартной библиотеке **нет** `dyn Clone`.

**Решение:** свой трейт с `Box<Self>`:

```rust
trait CloneBox {
    fn clone_box(&self) -> Box<dyn CloneBox>;
}
```

## Сводная таблица

| Трейт | `dyn`-совместим? | Почему |
|---|---|---|
| `Iterator` | ❌ Нет | `Self` в возврате (`Item` — ассоциированный) |
| `Clone` | ❌ Нет | `fn clone(&self) -> Self` |
| `Debug` | ✅ Да | `fn fmt(&self, ...) -> Result` |
| `Display` | ✅ Да | `fn fmt(&self, ...) -> Result` |
| `Default` | ❌ Нет | `fn default() -> Self` |
| `Read` | ✅ Да | `fn read(&mut self, ...) -> Result` |
| `Write` | ✅ Да | `fn write(&mut self, ...) -> Result` |
| `Fn` | ✅ Да (с `dyn Fn`) | Специальная обработка |

## Итог

- **Object safety** — правила, при которых трейт **можно** использовать как `dyn Trait`.
- **Главные ограничения:**
  - методы **не** принимают/возвращают `Self` **по значению**;
  - `Self` **не** в ассоциированных константах;
  - **нет** generic-методов;
  - **нет** `where Self: Sized` на **всём** трейте.
- **Причина:** vtable требует **фиксированных** слотов с **известными** сигнатурами.
- **Исключения:** `Box<Self>`, `&Self`, `&mut Self` — **разрешены**.
- **`where Self: Sized`** на **отдельном методе** — **разрешено** (метод не в vtable).
- **Классические `!dyn`:** `Clone`, `Iterator`, `Default`.
- **Классические `dyn`:** `Debug`, `Display`, `Read`, `Write`.
- **Правило:** если трейт возвращает `Self` по значению → **`!dyn`-совместим**.
