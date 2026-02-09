# Galaxy Clustering in the Mira-Titan Universe. I. Emulators for the Redshift Space Galaxy Correlation Function and Galaxy-Galaxy Lensing

**Song Huang (黄崧)** and collaborators

*Full author list:* Kwan, Juliana, Saito, Shun, Leauthaud, Alexie, Heitmann, Katrin, Habib, Salman, Frontiere, Nicholas, Guo, Hong, Huang, Song, Pope, Adrian, Rodriguéz-Torres, Sergio

*The Astrophysical Journal* (2023)

[DOI](https://doi.org/10.3847/1538-4357/acd92f) | [arXiv](https://arxiv.org/abs/2302.12379) | [ADS](https://ui.adsabs.harvard.edu/abs/2023ApJ...952...80K/abstract) | [PDF](https://ui.adsabs.harvard.edu/link_gateway/2023ApJ...952...80K/EPRINT_PDF)

**Citations:** 16

---

## Short Summary

We built the first cosmology-emulating framework—trained on 111 high-fidelity N-body simulations—that simultaneously and accurately models redshift-space clustering and galaxy-galaxy lensing across eight cosmological parameters, including neutrino mass and dynamical dark energy, enabling 2% precision on the growth rate when combined with CMB priors—a transformative leap for joint-probe cosmology with current galaxy surveys.

**中文：** 我们构建了首个宇宙学模拟框架——该框架基于111组高保真N体模拟进行训练，可同时、精确地建模红移空间中的星系成团性与星系-星系引力透镜效应，涵盖包括中微子质量与动力学暗能量在内的八个宇宙学参数；当结合CMB先验信息时，其对结构增长率的测量精度可达2%，这标志着当前星系巡天开展多探针联合宇宙学研究取得突破性进展。

## Detailed Summary

This work addresses a critical bottleneck in modern cosmological inference: the need for fast, accurate, and physically grounded theoretical predictions that span the full range of scales probed by galaxy surveys—especially where nonlinear structure formation, massive neutrinos, and evolving dark energy introduce complex, coupled dependencies. Prior emulators either lacked the cosmological breadth to jointly constrain neutrino mass and dynamical dark energy, or sacrificed fidelity on small scales where galaxy bias and halo physics imprint rich information. By leveraging the unprecedented scale and diversity of the Mira-Titan simulation suite—111 high-resolution N-body realizations spanning eight cosmological parameters, including ∑m<sub>ν</sub> up to 0.6 eV and w(a) parameterizations—the team bridges this gap with a novel, end-to-end framework rooted in halo occupation distribution (HOD) modeling. The result is not merely an interpolation tool, but a physically interpretable, differentiable mapping from cosmology and HOD parameters to observables, explicitly designed to support next-generation joint-probe analyses.

The methodology combines computational innovation with rigorous statistical validation: the emulators are trained on carefully calibrated HOD catalogs drawn from the full Mira-Titan ensemble, covering redshift-space two-point clustering (ξ(s,μ)), projected correlation functions (w<sub>p</sub>(r<sub>p</sub>)), and galaxy-galaxy lensing excess surface density (ΔΣ(R)). Crucially, the training incorporates realistic survey systematics—including CMASS-like selection, fiber collisions, and photometric redshift uncertainties—via forward-modeled mock catalogs. Emulator accuracy is validated through stringent leave-one-out cross-validation and blind recovery tests, demonstrating sub-percent residuals in ξ(s,μ) and ΔΣ(R) across 0.5 ≤ r ≤ 50 h<sup>−1</sup> Mpc, and robust extrapolation beyond the training prior boundaries. This level of precision—achieved without sacrificing physical transparency—is made possible by a hierarchical Gaussian process architecture that respects the known scaling behavior of clustering statistics under cosmological parameter shifts.

The key findings establish new benchmarks for precision cosmology with galaxy surveys. Using only the observables covered by the emulator—no external priors beyond the data model—the analysis recovers unbiased constraints on the growth rate fσ<sub>8</sub>(z=0.57) to 7% and σ<sub>8</sub> to 4.5% for a CMASS-like sample, demonstrating for the first time that small-scale lensing (R < 1 h<sup>−1</sup> Mpc) combined with redshift-space distortions breaks degeneracies between galaxy bias, neutrino mass, and structure growth more effectively than either probe alone. With the addition of a Planck-like CMB prior on H<sub>0</sub>, the growth rate uncertainty tightens to just 2%, rivaling constraints from much larger surveys while using only a single redshift slice. These results validate the emulator as a production-ready engine for current and upcoming missions—from DESI to Euclid—and underscore its unique capability to isolate the imprint of relativistic species and dark energy evolution in the nonlinear regime. By transforming computationally prohibitive simulations into agile, open-source inference tools, this work empowers the community to extract maximal cosmological insight from galaxy clustering and weak lensing—not as separate channels, but as a unified, self-consistent probe of cosmic structure formation.

### 中文版

本工作致力于解决现代宇宙学推断中的一项关键瓶颈：亟需一种快速、精确且具备坚实物理基础的理论预测方法，以覆盖星系巡天所探测的全尺度范围——尤其在非线性结构形成、大质量中微子及演化暗能量共同作用下，各类物理效应呈现出复杂而强耦合的依赖关系。此前的模拟器（emulator）或因宇宙学参数覆盖范围不足，难以同时约束中微子质量∑m<sub>ν</sub>与动力学暗能量状态方程w(a)；或为兼顾大尺度精度而牺牲小尺度保真度，从而丢失了星系偏差（galaxy bias）与晕物理（halo physics）所蕴含的丰富信息。本团队依托前所未有的Mira-Titan数值模拟套件——包含111组高分辨率N体模拟，横跨8个宇宙学参数（含∑m<sub>ν</sub>高达0.6 eV及多种w(a)参数化形式），成功弥合了这一鸿沟，构建了一种全新的端到端建模框架，其核心基于晕占有分布（HOD）理论。该成果不仅是一个插值工具，更是一种物理可解释、可微分的映射模型，能将宇宙学参数与HOD参数直接映射至可观测量，专为支持下一代多探针联合分析而设计。

该方法融合了计算技术创新与严格的统计验证：模拟器训练数据源自经精细校准的HOD星表，全部取自完整的Mira-Titan模拟集合，涵盖红移空间两点相关函数ξ(s,μ)、投影相关函数w<sub>p</sub>(r<sub>p</sub>)以及星系-星系弱引力透镜效应的面密度盈余ΔΣ(R)。尤为关键的是，训练过程通过前向建模（forward-modeled）的模拟星表，显式纳入了真实巡天系统误差——包括类CMASS选源、光纤碰撞（fiber collisions）及测光红移不确定性等。模拟器精度经严格的“留一法”交叉验证（leave-one-out cross-validation）与盲恢复测试（blind recovery tests）全面检验：在0.5 ≤ r ≤ 50 h<sup>−1</sup> Mpc范围内，ξ(s,μ)与ΔΣ(R)的残差均优于1%；且在训练先验边界之外仍展现出稳健的外推能力。此类高精度——在不牺牲物理透明度的前提下实现——得益于一种层级化高斯过程（hierarchical Gaussian process）架构，该架构严格尊重宇宙学参数变化下成团统计量已知的标度行为。

本研究的关键成果为星系巡天的精密宇宙学设定了新基准。仅利用模拟器所覆盖的观测量（除数据模型本身外不引入任何外部先验），针对类CMASS样本的分析即获得无偏的结构增长率fσ<sub>8</sub>(z=0.57)约束（精度7%）与σ<sub>8</sub>约束（精度4.5%）。这首次证实：小尺度透镜信号（R < 1 h<sup>−1</sup> Mpc）与红移空间畸变（RSD）的联合使用，对星系偏差、中微子质量和结构增长三者间简并性的破除能力，显著优于任一探针单独使用。若进一步加入类Planck CMB先验对哈勃常数H<sub>0</sub>进行约束，结构增长率的不确定性可进一步压缩至仅2%，其精度堪比更大规模巡天所得结果，却仅依赖单一红移切片。这些结果充分验证了该模拟器已具备工程化应用能力，可作为当前及未来重大巡天项目（从DESI到Euclid）的生产级分析引擎，并凸显其独特优势：在非线性尺度上清晰分离相对论性粒子（如中微子）与暗能量演化所产生的印记。本工作通过将原本计算代价高昂的数值模拟转化为敏捷、开源的推断工具，赋能整个天文界——不再将星系成团与弱引力透镜视为彼此独立的信道，而是将其整合为一个统一、自洽的宇宙结构形成探针，从而最大限度地从中提取宇宙学信息。

## Key Figure

*To be added in future version.* See `2023_kwan_galaxy_clustering_miratitan_universe_i_figures/` directory.
