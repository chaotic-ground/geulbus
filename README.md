# presguel

**날개셋(nalgaeset) 입력 설정과 호환되는, 순수 Rust로 작성하는 ibus 한글 입력기.**

> ⚠️ **개발 초기 (WIP).** 아직 동작하지 않습니다. 설계와 구현이 진행 중입니다.

---

## 무엇인가

[날개셋 한글 입력기](http://moogi.new21.org/)의 "입력 설정" XML(`nalgaeset.xml`)을
해석하여, Linux의 [ibus](https://github.com/ibus/ibus) 환경에서 **동일한 한글 입력
동작을 재현**하는 입력기 엔진입니다.

- **범용 해석기**: 특정 자판을 하드코딩하지 않고, 임의의 날개셋 설정 XML
  (`KeyTable` / `UnitMixTable` / `VirtualUnitTable` / `AutomataTable` /
  `FinalConvTable` 등)을 파싱·해석하는 것을 목표로 합니다.
- **순수 Rust**: `libibus`(C) 의존 없이 [`zbus`](https://github.com/dbus2/zbus)로
  ibus 데몬과 D-Bus를 직접 주고받습니다.
- **옛한글 완전 지원**: 첫가끝(U+1100 조합용 자모) 조합과 `FinalConvTable`을
  포함합니다. 옛한글 음절의 *표시*는 옛한글 OpenType 폰트(예: 함초롬)가 필요하며,
  이는 사용자 폰트 환경의 몫입니다. 엔진은 올바른 코드포인트를 commit 합니다.

## 배경

날개셋은 강력하지만 Windows 중심입니다. 같은 세벌식(및 임의 사용자 설정) 경험을
Linux ibus 위에서 그대로 쓰기 위해 만듭니다. 참고로 호환 대상 설정 파일은 이
저장소 바깥(사용자 환경)에 있는 사용자 소유 파일이며, 저장소에 포함하지 않습니다.

## 호환 대상 포맷

`EditContextSetting` (날개셋 입력 설정):

| 요소 | 역할 |
| --- | --- |
| `ShortcutTable` | 한/영 전환 등 특수키 동작 |
| `FinalConvTable` | 미완성/홑낱자 출력 시 조합용·옛 자모 → 호환 자모 변환 |
| `KeyTable` | 물리 키 → 자모 단위/문자 (값-식 언어) |
| `UnitMixTable` | 낱자 조합 규칙 (겹받침, 겹모음, 된소리, 갈마들이) |
| `VirtualUnitTable` | 가상 단위 |
| `AutomataTable` | 한글 조합 오토마타 (상태 전이 식) |
| `Extra` / `Bksp` | 백스페이스 동작 |

## 구조 (예정)

- **`presguel-core`** — 설정 파서 + 값-식 평가기 + 자모 모델 + 한글 오토마타.
  ibus와 무관한 순수 라이브러리. 테스트 벡터로 동작을 검증합니다.
- **`presguel-ibus`** — `zbus` 기반 ibus 프런트엔드(Factory / Engine),
  preedit·commit, 한/영 전환.

## 진행 상황

- [ ] 날개셋 포맷·식 언어·오토마타 해석 명세
- [ ] 설정 파일 역공학 → 동작 명세·테스트 벡터(오라클)
- [ ] `presguel-core`: 파서 / 식 평가 / 자모 / 오토마타
- [ ] `presguel-ibus`: zbus 프런트엔드
- [ ] 옛한글(첫가끝) 조합·출력
- [ ] 설치 / 패키징(ibus 컴포넌트 등록)

## 빌드 / 설치

추후 작성합니다.

## 라이선스

이 저장소는 **공개(public)되어 있으나 오픈소스가 아닙니다.** 자세한 내용은
[`LICENSE`](./LICENSE)를 참고하세요. 요지: 모든 권리 보유(All rights reserved),
저작권자의 사전 허가 없이는 사용·복제·수정·배포할 수 없습니다. 라이선스는 추후
확정될 수 있습니다.
