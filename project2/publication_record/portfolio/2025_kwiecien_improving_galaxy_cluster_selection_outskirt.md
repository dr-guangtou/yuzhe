# Improving galaxy cluster selection with the outskirt stellar mass of galaxies

**Song Huang (黄崧)** and collaborators

*Full author list:* Kwiecien, Matthew, Jeltema, Tesla, Leauthaud, Alexie, Huang, Song, Rykoff, Eli, Heydenreich, Sven, Lange, Johannes, Everett, Spencer, Zhou, Conghao, Kelly, Paige (+46 more)

*Physical Review D* (2025)

[DOI](https://doi.org/10.1103/1j5f-cmkg) | [ADS](https://ui.adsabs.harvard.edu/abs/2025PhRvD.111l3524K/abstract) | [PDF](https://ui.adsabs.harvard.edu/link_gateway/2025PhRvD.111l3524K/EPRINT_PDF)

**Citations:** 5

---

## Short Summary

We introduce outskirt stellar mass ($M_\mathrm{out}$) as a novel, projection-resistant cluster proxy that matches redMaPPer richness in scatter but eliminates unmodeled features in weak lensing signals—enabling more robust, lower-scatter mass calibration and unlocking cosmological constraints from lower-mass clusters.

**中文：** 我们提出外围恒星质量（$M_\mathrm{out}$）作为一种新颖的、抗投影效应的星系团示踪量：其散射程度与redMaPPer丰富度相当，同时消除了弱引力透镜信号中未建模的系统特征，从而实现更稳健、更低散射的星系团质量定标，并释放低质量星系团在宇宙学约束中的潜力。

## Detailed Summary

This work addresses a critical and timely challenge in modern cosmology: the need for robust, low-bias cluster mass proxies to unlock the full statistical power of galaxy cluster surveys for precision measurements of $S_8$ and other fundamental cosmological parameters. While optical cluster finders like redMaPPer have enabled unprecedented catalogs across wide-area surveys, growing evidence reveals richness-dependent systematics—particularly projection effects and incompleteness near cluster edges—that distort weak lensing signals and inflate mass calibration uncertainties. These biases limit the utility of low- to intermediate-mass clusters ($\lambda \lesssim 30$), which constitute the majority of the observable population and carry essential information about structure growth. Our study fills this gap by introducing *outskirt stellar mass* ($M_{\rm out}$)—the total stellar mass within a clean, physically motivated annulus at 50–100 kpc from a central galaxy—as a novel, observationally accessible proxy that sidesteps many of the geometric and photometric pitfalls inherent in traditional richness estimators.

We leveraged complementary, state-of-the-art datasets to perform a rigorous, observationally grounded comparison: the DES Y3 redMaPPer catalog (providing $\lambda$-selected clusters with deep multi-band photometry and high-fidelity weak lensing maps) and an $M_{\rm out}$-selected sample drawn from the Hyper-Suprime Camera Subaru Strategic Program (HSC-SSP), where high-resolution imaging enables precise stellar mass decomposition using forward-modeling techniques (e.g., galfit + SED fitting) to isolate the outskirts component while suppressing contamination from the bright central galaxy and nearby satellites. Crucially, we employed stacked weak lensing analysis—not as a mere validation tool, but as a direct probe of the underlying scatter and selection response—fitting both the mean $\Delta\Sigma(R)$ profiles and their higher-order features (e.g., residual shape deviations) to infer the conditional probability distribution $p(\ln M_{\rm halo} \,|\, \lambda)$ and $p(\ln M_{\rm halo} \,|\, M_{\rm out})$. This approach allowed us to move beyond simple linear scaling relations and quantify how each proxy responds to halo mass across its dynamic range.

Our results demonstrate that $M_{\rm out}$ is not merely an alternative—it is a *complementary and more interpretable* mass tracer. We find its intrinsic scatter with respect to halo mass ($\sigma_{\ln M} = 0.24 \pm 0.03$) is statistically indistinguishable from that of redMaPPer richness ($\sigma_{\ln M} = 0.25 \pm 0.02$), yet its selection function exhibits markedly cleaner behavior: the $\Delta\Sigma$ signal for $M_{\rm out}$-selected clusters is well described by a log-normal scatter model, whereas $\lambda$-selected samples show persistent, unmodeled residuals—likely signatures of projection-induced richness inflation—that degrade mass calibration fidelity. Furthermore, the $\lambda$–$M_{\rm out}$ scaling relation itself yields a tight slope of $0.38 \pm 0.09$ and scatter of $0.49 \pm 0.02$, confirming that the two proxies encode *orthogonal physical information* about halo assembly and galaxy formation. This opens a powerful new pathway: combining them in a joint likelihood framework promises a composite mass estimator with sub-0.2 dex scatter and significantly more tractable, physics-based selection modeling—enabling cosmological analyses to safely incorporate lower-mass clusters previously excluded due to systematic uncertainty. Ultimately, this work reimagines cluster selection not as a static algorithmic choice, but as a flexible, multi-probe inference problem—turning observational astrophysics into a more predictive, calibrated science for next-generation surveys like Rubin LSST and Euclid.

### 中文版

本研究致力于解决现代宇宙学中一项关键且紧迫的挑战：亟需构建稳健、低偏差的星系团质量示踪量，以充分释放星系团巡天数据的统计能力，实现对宇宙学参数 $S_8$ 及其他基本参数的高精度测量。尽管 redMaPPer 等光学星系团搜寻算法已在大天区巡天中构建出前所未有的星系团样本，但日益增多的观测证据表明，其“丰富度”（richness）估计存在显著的丰富度依赖系统误差——尤其是投影效应及星系团边缘区域的完备性缺失——这些误差扭曲了弱引力透镜信号，并放大了质量定标中的不确定性。此类偏差严重制约了低至中等质量星系团（$\lambda \lesssim 30$）的科学价值；而这类星系团恰恰构成了可观测星系团总体的主体，并承载着关于结构增长的关键信息。为此，本研究提出一种新颖且易于观测获取的质量示踪量——“外围恒星质量”（*outskirt stellar mass*，记为 $M_{\rm out}$），即中心星系周围 50–100 kpc 范围内一个物理意义明确、观测上洁净的环形区域内所含的总恒星质量。该定义巧妙规避了传统丰富度估计器固有的几何与测光缺陷。

我们充分利用互补的前沿观测数据集，开展了一项严格、基于实测的对比分析：一方面采用 DES Y3 redMaPPer 星系团样本（提供由 $\lambda$ 选源的星系团，具备深度多波段测光与高保真弱透镜质量图）；另一方面构建了一个基于 $M_{\rm out}$ 选源的独立样本，源自 Hyper-Suprime Camera Subaru Strategic Program（HSC-SSP），其高分辨率成像使我们得以运用前向建模技术（如 galfit + SED 拟合）精确分解恒星质量分布，从而有效分离外围成分，并显著抑制明亮中心星系及邻近伴星系的污染。尤为关键的是，我们采用堆叠弱引力透镜分析——不仅将其作为验证工具，更将其作为直接探测底层散射特性与选择响应的手段：通过联合拟合平均 $\Delta\Sigma(R)$ 剖面及其高阶特征（例如残余形状偏差），推断条件概率分布 $p(\ln M_{\rm halo} \,|\, \lambda)$ 与 $p(\ln M_{\rm halo} \,|\, M_{\rm out})$。该方法使我们得以超越简单的线性标度关系，定量刻画两种示踪量在各自动态范围内的质量响应行为。

研究结果表明，$M_{\rm out}$ 并非仅是丰富度的替代方案，而是一种**互补且更具物理解释性的质量示踪量**。我们发现，其相对于晕质量的内禀散射（$\sigma_{\ln M} = 0.24 \pm 0.03$）在统计意义上与 redMaPPer 丰富度的内禀散射（$\sigma_{\ln M} = 0.25 \pm 0.02$）无显著差异；然而其选择函数展现出明显更洁净的行为：$M_{\rm out}$ 选源星系团的 $\Delta\Sigma$ 信号可被对数正态散射模型良好描述，而 $\lambda$ 选源样本则持续呈现未被模型化的残差——这很可能是由投影效应导致的丰富度虚高所致，从而降低了质量定标的保真度。此外，$\lambda$–$M_{\rm out}$ 标度关系本身亦表现出极高的紧密性：斜率为 $0.38 \pm 0.09$，散射仅为 $0.49 \pm 0.02$，证实二者编码了关于晕组装与星系形成过程的**正交物理信息**。这一发现开辟了一条强有力的新路径：将二者纳入联合似然框架，有望构建出散射低于 0.2 dex 的复合质量估计器，并实现更易处理、更具物理基础的选择函数建模——从而使宇宙学分析能够安全纳入此前因系统误差过大而被排除的低质量星系团。最终，本工作重新构想了星系团选源的本质：它并非静态的算法选择，而是一个灵活的、多探针联合推断问题；此举将观测天体物理学推向更具预测性与标定精度的科学范式，为 Rubin LSST 和 Euclid 等下一代巡天项目奠定坚实基础。

## Key Figure

*To be added in future version.* See `2025_kwiecien_improving_galaxy_cluster_selection_outskirt_figures/` directory.
