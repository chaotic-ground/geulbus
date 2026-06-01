# geulbus-core

날개셋(Nalgaeset) 호환 한글 입력 **설정(`layout.xml`, 종합 설정 `.set`)을 해석해
키 입력을 한글로 조합**하는, 프론트엔드에 의존하지 않는 순수 Rust 엔진입니다.
IBus 프론트엔드는 별도 크레이트 [`geulbus`](https://github.com/chaotic-ground/geulbus)
가 이 라이브러리를 감싼 것이고, 다른 프론트엔드(Fcitx5, Windows TSF, wasm 등)도
같은 방식으로 이 위에 올릴 수 있습니다.

- 라이선스: **MIT OR Apache-2.0**
- 자모/음절 유니코드 계층은 [`hanmo`](https://crates.io/crates/hanmo) 크레이트에 분리.
- 설정 XML 형식 사양: [nalgaeset-reverse-spec](https://chaotic-ground.github.io/nalgaeset-reverse-spec/).

## 프론트엔드에서 쓰는 법 (계약)

```rust
use geulbus_core::{Config, Engine};

// 1. 날개셋 설정 XML 파싱 → 입력 항목 하나를 Layout 으로 컴파일
let cfg = Config::parse(&xml)?;
let layout = cfg.compile(0)?;            // 0 = 첫 입력 항목
let mut engine = Engine::new(layout);

// 2. 키마다: 플랫폼 키를 ASCII(0x21..=0x7E)로 매핑해 press
//    (헬퍼: geulbus_core::us_qwerty_ascii / evdev_to_ascii)
let out = engine.press(b'k', false);     // KeyOutcome
// out.commit   : 응용에 확정 입력할 문자열
// out.preedit  : 조합 중 표시
// out.consumed : 엔진이 이 키를 소비했는지(false 면 원래 키를 응용에 넘김)
// out.delete_before : 커서 앞 확정 글자 N개 삭제 요청(surrounding-text)

let _ = engine.backspace();              // 백스페이스
let tail = engine.flush();               // 조합 종료 시 남은 글자 확정
engine.reset();                          // 포커스 아웃 등에서 초기화
```

프론트엔드가 할 일은 (a) 플랫폼 키심을 ASCII 글쇠코드로 매핑, (b) `press`/`backspace`
결과의 `commit`/`preedit`/`delete_before` 를 플랫폼 API로 반영, 그게 전부입니다.
단축글쇠 값-식 평가가 필요하면 `geulbus_core::expr` 를 쓸 수 있습니다. 실제 동작
예시는 `geulbus` 크레이트(IBus 프론트엔드)가 레퍼런스 구현입니다.
