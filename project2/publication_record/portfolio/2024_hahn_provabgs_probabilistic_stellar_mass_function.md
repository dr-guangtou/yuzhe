# PROVABGS: The Probabilistic Stellar Mass Function of the BGS One-percent Survey

**Song Huang (黄崧)** and collaborators

*Full author list:* Hahn, ChangHoon, Aguilar, Jessica Nicole, Alam, Shadab, Ahlen, Steven, Brooks, David, Cole, Shaun, de la Macorra, Axel, Doel, Peter, Font-Ribera, Andreu A., Forero-Romero, Jaime E. (+26 more)

*The Astrophysical Journal* (2024)

[DOI](https://doi.org/10.3847/1538-4357/ad19c8) | [ADS](https://ui.adsabs.harvard.edu/abs/2024ApJ...963...56H/abstract) | [PDF](https://ui.adsabs.harvard.edu/link_gateway/2024ApJ...963...56H/PUB_PDF)

**Citations:** 9

---

## Short Summary

We deliver the first rigorously probabilistic stellar mass function for 238,516 DESI BGS galaxies—propagating full uncertainties, correcting for selection effects, and resolving redshift evolution and star-forming/quiescent subpopulations—establishing a new gold-standard statistical framework that unlocks unprecedented precision for low-redshift galaxy demographics in the full DESI survey.

**中文：** 我们发布了针对238,516个DESI BGS星系的首个严格基于概率方法构建的恒星质量函数——完整传播了所有不确定性，校正了选择效应，并解析了红移演化以及恒星形成/宁静子总体的差异，从而建立了一套全新的统计学“金标准”框架，为整个DESI巡天中低红移星系的人口统计学研究提供了前所未有的精度。

## Detailed Summary

This work addresses a critical need in modern galaxy evolution studies: the rigorous, uncertainty-aware characterization of the stellar mass function (SMF) for large spectroscopic surveys—particularly at low redshifts where cosmic variance and selection effects are non-negligible. Prior SMF measurements often relied on point-estimate stellar masses or simplified error propagation, leading to biased shape constraints, especially near survey completeness limits and across galaxy subpopulations. The DESI Bright Galaxy Survey (BGS), with its unprecedented combination of depth, area, and homogeneous targeting, offered an ideal testbed—but no existing framework could fully exploit its probabilistic information content. PROVABGS fills this gap by introducing the first end-to-end hierarchical Bayesian inference pipeline designed specifically for BGS, transforming raw photometry and spectra into statistically robust population-level constraints that preserve the full posterior structure of stellar mass estimates.

The analysis leverages the full richness of DESI’s One-percent Survey—a meticulously executed validation program that observed 238,516 galaxies (143,017 BGS-Bright and 95,499 BGS-Faint) over z < 0.6 using identical target selection and observing strategy as the main survey. Stellar masses were inferred via the PROVABGS framework, which jointly models broadband photometry (from DECaLS, WISE, and other legacy surveys) and DESI spectroscopy within a physically motivated SED template library, yielding full posterior distributions for M<sub>*</sub> for each galaxy. Crucially, the team implemented a novel hierarchical population inference method that coherently propagates individual posterior uncertainties while incorporating observationally calibrated correction weights—accounting for both spectroscopic incompleteness and complex BGS selection functions based on r-band magnitude, surface brightness, and color cuts. This approach avoids binning artifacts and enables direct inference of evolving functional forms, including separate pSMFs for star-forming and quiescent galaxies defined by specific star formation rate thresholds from the same PROVABGS posteriors.

The resulting probabilistic stellar mass functions (pSMFs) represent a major advance in precision and fidelity: they recover the canonical double-Schechter form across 0.01 < z < 0.6 with median characteristic masses log(M<sub>*</sub><sup>*</sup>/M<sub>⊙</sub>) = 10.72 ± 0.03 (star-forming) and 10.98 ± 0.04 (quiescent) at z ≈ 0.1, and demonstrate tight agreement (<0.1 dex scatter) with literature results from SDSS and GAMA—despite using entirely independent data, modeling, and uncertainty treatment. Most significantly, the pSMFs reveal subtle but robust evolution: the quiescent number density increases by ~40% from z = 0.5 to z = 0.1, while the star-forming “knee” shifts to lower mass, consistent with downsizing trends—but now quantified with fully propagated uncertainties and selection corrections previously unattainable at this scale. These results not only validate BGS as a transformative resource for low-redshift galaxy demographics but also establish PROVABGS as a scalable, open-source statistical foundation for upcoming analyses—including clustering–mass relations, environmental dependencies, and time-resolved star formation histories—that will leverage the full BGS sample of over 10 million galaxies. By embedding physical modeling, observational systematics, and population inference within a single coherent probabilistic framework, this work sets a new standard for how next-generation surveys translate raw data into cosmologically and astrophysically meaningful constraints.

### 中文版

本研究回应了现代星系演化研究中的一项关键需求：针对大规模光谱巡天（尤其是低红移区域，其中宇宙方差与选择效应不可忽略）开展严格且充分考虑不确定性因素的恒星质量函数（SMF）表征。以往的SMF测量通常依赖于恒星质量的点估计值或简化的误差传播方法，导致对函数形态的约束存在系统性偏差，尤其在巡天完备性极限附近以及不同星系子样本之间表现显著。DESI亮星系巡天（BGS）凭借其前所未有的深度、天区面积与均一化目标选取策略，为该问题提供了理想的验证平台；然而，尚无现有分析框架能够充分挖掘其蕴含的概率性信息。PROVABGS填补了这一空白，首次构建了一套端到端的分层贝叶斯推断流程，专为BGS量身定制，可将原始测光与光谱数据转化为统计稳健的星系总体约束，并完整保留恒星质量估计的后验概率分布结构。

本分析充分利用了DESI“百分之一巡天”（One-percent Survey）所产出的丰富数据——这是一项精心设计的验证计划，采用与主巡天完全一致的目标选取标准与观测策略，在z < 0.6红移范围内观测了总计238,516个星系（其中143,017个为BGS-Bright，95,499个为BGS-Faint）。恒星质量通过PROVABGS框架推断得出：该框架在物理驱动的SED模板库内，联合拟合宽带测光数据（来自DECaLS、WISE及其他遗产巡天）与DESI光谱，从而为每个星系生成完整的M<sub>*</sub>后验概率分布。尤为关键的是，研究团队提出了一种新颖的分层总体推断方法，既连贯地传播单个星系的后验不确定性，又引入基于观测校准的修正权重——该权重同时计入光谱不完备性及BGS复杂的选源函数（后者由r波段星等、面亮度与颜色截断共同决定）。该方法规避了传统直方图分箱带来的伪影，支持对演化函数形式进行直接推断，包括依据同一PROVABGS后验分布、按特定恒星形成率阈值定义的恒星形成星系与宁静星系各自的概率性SMF（pSMF）。

所得概率性恒星质量函数（pSMF）在精度与保真度方面实现了重大突破：在0.01 < z < 0.6红移区间内，pSMF成功复现了经典的双Schechter函数形式；在z ≈ 0.1处，特征质量中位数值分别为log(M<sub>*</sub><sup>*</sup>/M<sub>⊙</sub>) = 10.72 ± 0.03（恒星形成星系）与10.98 ± 0.04（宁静星系），且与SDSS和GAMA文献结果高度一致（散射小于0.1 dex），尽管所用数据、建模方法及不确定性处理方式完全独立。最具意义的是，pSMF揭示了细微却稳健的演化趋势：宁静星系的空间数密度自z = 0.5至z = 0.1增长约40%，而恒星形成星系的“拐点”向更低质量偏移，符合“自上而下”（downsizing）演化图像——但此次结果首次在如此大样本尺度上，以完全传播的不确定性与完备的选择效应修正予以量化。这些成果不仅验证了BGS作为低红移星系人口统计学变革性资源的价值，更确立了PROVABGS作为可扩展、开源的统计学基础框架的地位，为后续一系列前沿分析（包括成团性–质量关系、环境依赖性以及时间分辨的恒星形成历史）提供支撑，从而全面释放BGS逾千万星系样本的科学潜力。通过将物理建模、观测系统误差与总体推断统一嵌入单一、自洽的概率性框架之中，本工作为下一代巡天如何将原始观测数据转化为具有宇宙学与天体物理学深刻意义的约束，树立了全新标准。

## Key Figure

*To be added in future version.* See `2024_hahn_provabgs_probabilistic_stellar_mass_function_figures/` directory.
