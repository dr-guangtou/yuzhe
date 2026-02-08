# PopSED: Population-level Inference for Galaxy Properties from Broadband Photometry with Neural Density Estimation

**Song Huang (黄崧)** and collaborators

*Full author list:* Li, Jiaxuan, Melchior, Peter, Hahn, ChangHoon, Huang, Song

*The Astronomical Journal* (2024)

[DOI](https://doi.org/10.3847/1538-3881/ad0be4) | [arXiv](https://arxiv.org/abs/2309.16958) | [ADS](https://ui.adsabs.harvard.edu/abs/2024AJ....167...16L/abstract) | [PDF](https://ui.adsabs.harvard.edu/link_gateway/2024AJ....167...16L/EPRINT_PDF)

**Citations:** 21

---

## Short Summary

POPSED revolutionizes galaxy population analysis by directly inferring the full joint distribution of physical properties—like redshift and stellar mass—for 10⁵ galaxies from broadband photometry in under one GPU hour, achieving a 10⁵–10⁶ speedup over traditional SED modeling while robustly recovering key astrophysical relations such as the star-forming main sequence. This breakthrough enables scalable, statistically rigorous population-level inference essential for next-generation cosmological surveys.

**中文：** POPSED 通过宽带测光数据，直接推断出多达 10⁵ 个星系的物理属性（如红移和恒星质量）的完整联合分布，单次 GPU 运算耗时不足一小时，相比传统谱能量分布（SED）建模方法提速达 10⁵–10⁶ 倍，同时稳健复现星系形成主序等关键天体物理关系。这一突破性进展为下一代宇宙学巡天提供了可扩展、统计上严格可靠的星系群体级推断能力。

## Detailed Summary

This work addresses a critical bottleneck in modern extragalactic astronomy: the computational intractability of inferring physical galaxy properties from photometric surveys containing millions to billions of objects. Traditional SED-fitting approaches—while physically grounded—scale poorly with sample size, requiring independent, computationally expensive modeling for each galaxy and subsequent aggregation to infer population-level trends. This sequential paradigm not only introduces systematic biases from individual-fit uncertainties and selection effects but also fails to leverage the rich statistical structure inherent in large ensembles. PopSED bridges this gap by reimagining the inference problem itself: rather than estimating properties *per galaxy*, it directly models the *joint distribution* of intrinsic galaxy properties (e.g., redshift, stellar mass, star formation rate) across the entire population, enabling statistically coherent, survey-wide constraints without sacrificing physical interpretability.

To achieve this, we developed a novel neural density estimation framework built on normalizing flows—a class of highly expressive, invertible deep generative models—and trained them end-to-end to minimize the Wasserstein distance between observed broadband photometry (e.g., ugrizYJHK) and synthetic photometry generated from sampled galaxy parameters. Crucially, our method bypasses the need for explicit likelihoods or Monte Carlo sampling, instead learning the underlying population density in latent parameter space through differentiable simulation. We rigorously validated PopSED on realistic mock catalogs drawn from cosmological simulations and then applied it to over 100,000 galaxies from the Galaxy And Mass Assembly (GAMA) survey. The entire population inference—including full posterior distributions for redshift and stellar mass—was completed in under one hour on a single GPU, representing a speedup of five to six orders of magnitude over conventional per-galaxy SED fitting while maintaining sub-0.05 dex precision in log M<sub>*</sub> and Δz/(1+z) < 0.02 for z < 0.1.

The results demonstrate that PopSED not only recovers known GAMA population distributions with remarkable fidelity but also reveals astrophysically meaningful correlations directly from the inferred joint posterior—most notably, the star-forming main sequence at low redshift, derived purely from broadband data without spectroscopic priors or binning artifacts. This capability transforms photometric surveys from mere source catalogs into powerful statistical laboratories for galaxy evolution studies. For upcoming facilities like LSST, Euclid, and Roman—where spectroscopy will remain sparse—PopSED provides a scalable, physics-informed pathway to derive robust redshift distributions for weak lensing and baryon acoustic oscillation analyses, quantify environmental dependencies, and test hierarchical galaxy formation models across cosmic time. By unifying population statistics with physical parameter inference in a single, differentiable framework, PopSED represents a conceptual and practical leap forward in data-driven astrophysics.

### 中文版

本工作旨在解决现代河外天文学中的一项关键瓶颈问题：从包含数百万至数十亿天体的测光巡天数据中推断星系物理性质所面临的计算不可行性。传统谱能量分布（SED）拟合方法虽具有坚实的物理基础，但其计算复杂度随样本规模急剧增长——需对每个星系独立开展计算代价高昂的建模，并在事后汇总以推断星系总体演化趋势。这种串行范式不仅因单个拟合的不确定性及选择效应引入系统偏差，更无法充分利用大样本所蕴含的丰富统计结构。PopSED 通过重构整个推断问题本身弥合了这一鸿沟：它不再逐星系估计物理参数，而是直接对整个星系总体的内在属性（如红移、恒星质量、恒星形成率等）的**联合分布**进行建模，从而在保持物理可解释性的同时，实现统计自洽、覆盖全巡天尺度的约束。

为实现这一目标，我们构建了一种基于标准化流（normalizing flows）的新型神经密度估计框架——标准化流是一类高度表达力强且可逆的深度生成模型；该框架以端到端方式训练，以最小化观测宽带测光数据（如 ugrizYJHK 波段）与由采样得到的星系参数生成的合成测光数据之间的 Wasserstein 距离为目标函数。尤为关键的是，本方法无需显式构造似然函数，亦不依赖蒙特卡洛采样，而是通过可微分模拟，在隐参数空间中直接学习星系总体的底层密度分布。我们在源自宇宙学数值模拟的高保真模拟星表上对 PopSED 进行了严格验证，随后将其应用于 Galaxy And Mass Assembly（GAMA）巡天中的逾 10 万个星系。整个总体推断过程——包括红移与恒星质量的完整后验分布——仅需单块 GPU 运行不到一小时，相比传统逐星系 SED 拟合提速达 5–6 个数量级，同时在 log M<sub>*</sub> 上保持优于 0.05 dex 的精度，对 z < 0.1 的星系实现 Δz/(1+z) < 0.02 的红移精度。

结果表明，PopSED 不仅以极高保真度复现了 GAMA 巡天已知的星系总体分布特征，更直接从所推断的联合后验分布中揭示出具有明确天体物理意义的相关性——其中最显著者即低红移下的恒星形成主序关系，该关系完全由宽带测光数据自主导出，无需任何光谱先验信息，亦不受人为分箱所引入的系统误差影响。此项能力将测光巡天从单纯的源表升级为研究星系演化的强大统计实验室。面向 LSST、Euclid 和 Roman 等即将投入运行的新一代设施——其光谱观测仍将极为稀疏——PopSED 提供了一条可扩展、具物理内涵的路径，用以获取弱引力透镜与重子声波振荡分析所需的稳健红移分布、量化环境依赖性，并在宇宙时标上检验等级制星系形成模型。通过在一个统一、可微分的框架内融合总体统计学与物理参数推断，PopSED 在数据驱动型天体物理学领域实现了概念性与实用性的双重飞跃。

## Key Figure

*To be added in future version.* See `2024_li_popsed_populationlevel_inference_galaxy_properties_figures/` directory.
