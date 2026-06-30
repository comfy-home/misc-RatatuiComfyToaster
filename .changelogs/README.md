# Changelog History

Newest archived changelogs first. When multiple archived files represent the same version, only the newest archive is included here.

## Changelog `v0.4.3` <sup><div align="end">🗓️ 2026-06-30</div></sup>

### 💥 💥 💥 This Release's Top Picks ...  💥 💥 💥

<sup>💬 Intro:</sup>  
<sup>_This release does not bring any new features, just dependecy updates._</sup>  

#### **1. &nbsp;&nbsp;&nbsp;Updated:**
- `ratatui`: 0.30.1 -> 0.30.2
- `anyhow` (dev-dep): 1.0.102 → 1.0.103
- `tachyonfx` (dev-dep): 0.17.0 → 0.25


<sub>...  🎉 Enjoy!</sub>

<br>

### 🔧 Maintenance

* CG app version bump to v0.4.3 <sub><sup><sup>_d256070_</sup></sup></sub>

* bump anyhow to 1.0.103, ratatui to 0.30.2, and tachyonfx to 0.25 <sub><sup><sup>_d6fe8f9_</sup></sup></sub>

* reformat ToastMouseButton::Right match arm <sub><sup><sup>_f296d44_</sup></sup></sub>

### ℹ️ Documentation

* README update <sub><sup><sup>_2f6372d_</sup></sup></sub>

### 📝 Other

* Merge pull request #1 (via ComfyGit) <sub><sup><sup>_a0399a8_</sup></sup></sub>

* Merge branch 'main' into 0.4.x <sub><sup><sup>_9690d0c_</sup></sup></sub>

* Merge pull request #2 (via ComfyGit) <sub><sup><sup>_cc0e123_</sup></sup></sub>

* Merge pull request #3 (via ComfyGit) <sub><sup><sup>_92fc2c9_</sup></sup></sub>

---

## Changelog `v0.4.1` <sup><div align="end">🗓️ 2026-06-02</div></sup>

### 💥 💥 💥 This Release's Top Picks ...  💥 💥 💥

#### **1. &nbsp;&nbsp;&nbsp;Feel free to **ignore** this v0.4.1 release...**
- if you previously had v0.4.0 this release does not bring any functional improvements
- it contains just updated documentation


<sub>...  🎉 Enjoy!</sub>

<br>

### 🔧 Maintenance

* CG app version bump to v0.4.1 <sub><sup><sup>_e5c46d3_</sup></sup></sub>

---

## Changelog `v0.4.0` <sup><div align="end">🗓️ 2026-05-16</div></sup>

### 💥 💥 💥 This Release's Top Picks ...  💥 💥 💥

#### **1. &nbsp;&nbsp;&nbsp;Add a TITLE to your toasts!**
- Learn more about this feature [HERE](#%EF%B8%8F-optional-title-line)
- You can set them up with your own config, or use...

#### **2. &nbsp;&nbsp;&nbsp;Toast title PRESETS!**
- See details in documentation sections:
    - [Configuration options](#%EF%B8%8F-toast-title-presets)
    - [Examples](#toast-titile-preset-examples)


<sub>...  🎉 Enjoy!</sub>

<br>

### 🧩 Features

* add title support to ToastBuilder and update copy text functionality   _(baea963)_

* enhance Toast widget to include optional title rendering above the message   _(b0421e0)_

* refactor Toast widget to use ToastTitle struct and enhance title rendering functionality   _(b579775)_

* implement ToastTitle struct with layout options and rendering logic for toast titles   _(9085399)_

* add title module and expose it in the library for enhanced toast functionality   _(ac4edce)_

* enhance ToastBuilder with new title configuration methods and improve title handling in toast area calculations   _(3d3c756)_

* update toast vertical chrome calculation to include title padding and adjust related tests   _(f066474)_

* add functions for title presence and padding calculations in toast rendering   _(4dcb06a)_

* refine toast rendering by adjusting title padding and separator color handling   _(57fa2d7)_

* introduce preset functionality in ToastBuilder for customizable title layouts   _(44b7425)_

* add presets module and expose it in the library for enhanced customization options   _(06f5348)_

* introduce ToastPreset enum for predefined toast title layouts and styling options   _(4fb6d9b)_

### 🔧 Maintenance

* CG app version bump to v0.3.3   _(3fc5989)_

* CG app version bump to v0.4.0   _(c868695)_

### ℹ️ Documentation

* add new example images for toast presets   _(d2e7fa8)_

* update README with optional title line feature for toasts, presets, and usage examples   _(02a2162)_

* added preset examples   _(a6e748f)_

### ♻️ Refactor

* improve title rendering logic in Toast widget by adjusting layout calculations and highlight behavior   _(f652997)_

* streamline toast area calculation by introducing ToastLayoutParams struct for improved readability and maintainability   _(2a0b802)_

* simplify conditional checks in toast rendering logic for improved clarity   _(d2fbd25)_

### 📝 Other

* Merge pull request #6 (via ComfyGit)   _(d8e30c1)_

---

## Changelog `v0.3.2` <sup><div align="end">🗓️ 2026-05-13</div></sup>

### 💥 💥 💥 This Release's Top Picks ...  💥 💥 💥

#### **1. &nbsp;&nbsp;&nbsp;Expiration Progress Bar**
- Now your timed toasts can display an optional expiry bar
- Available are 3 styles:
    - FullBlock: ████
    - HalfBlock: ▄▄▄▄ 
    - Minimal: ____
- See documentation for more info...

#### **2. &nbsp;&nbsp;&nbsp;Toasts now support two border modes:**
- `ToastBorderMode::SideRails` keeps the original left/right look
- `ToastBorderMode::Full` renders a full box border for stronger separation
    - It's useful mainly with `Center` positioned toasts


<sub>...  🎉 Enjoy!</sub>

<br>

### 🧩 Features

* Introduce ToastBorderMode and default progress bar settings for toasts   _(83f63c6)_

* Enhance Toast widget with customizable border modes and optional progress bar rendering   _(e5dc454)_

* Add ToastProgressBarStyle for enhanced customization of toast progress bars   _(ed107d6)_

* Extend Toast widget to include progress bar style customization and add corresponding tests   _(9e5066d)_

### 🔧 Maintenance

* CG app version bump to v0.3.2   _(3f81c5b)_

* formatting fix   _(788b9cb)_

### 📝 Other

* Merge branch 'main' into v0.3.2-dev--render-enh   _(4caa834)_

* Merge pull request #5 (via ComfyGit)   _(7a69389)_

---

## Changelog `v0.3.1` <sup><div align="end">🗓️ 2026-05-12</div></sup>

### 🔧 Maintenance

* CG app version bump to v0.3.1   _(be6caae)_

### ♻️ Refactor

* Improve toast queue management by removing expired toasts and optimizing area calculations   _(33d72fe)_

* Optimize toast expiration check in queue management   _(67ace50)_

### 📝 Other

* Merge pull request #2 (via ComfyGit)   _(9073851)_

* Merge pull request #3 (via ComfyGit)   _(7f57b6d)_

* Merge pull request #4 (via ComfyGit)   _(041653d)_

---

## Changelog `v0.2.2` <sup><div align="end">🗓️ 2026-05-12</div></sup>

### 🧩 Features

* Implement area avoidance for toast positioning to prevent overlap   _(1c85859)_

### 🔧 Maintenance

* Update version to 0.2.2 and add README with detailed features and usage examples   _(85d7f2b)_

* Update Cargo.toml and README for version 0.2.2, changing license format and refining dependency specifications   _(a740600)_

### ♻️ Refactor

* Improve code readability by formatting and restructuring logic in engine.rs   _(ca58d0b)_

* Optimize sorting logic for toast position blockers   _(6b78c89)_

### 🧪 Tests

* Add test for toast position adjustment to avoid overlap with blockers   _(fa9b955)_



---
... ✨ made with [ComfyGit](https://github.com/comfy-home/ComfyGit)