# Anomaly detection in Hyper Suprime-Cam galaxy images with generative adversarial networks

**Song Huang (黄崧)** and collaborators

*Full author list:* Storey-Fisher, Kate, Huertas-Company, Marc, Ramachandra, Nesar, Lanusse, Francois, Leauthaud, Alexie, Luo, Yifei, Huang, Song, Prochaska, J. Xavier

*Monthly Notices of the Royal Astronomical Society* (2021)

[DOI](https://doi.org/10.1093/mnras/stab2589) | [ADS](https://ui.adsabs.harvard.edu/abs/2021MNRAS.508.2946S/abstract) | [PDF](https://ui.adsabs.harvard.edu/link_gateway/2021MNRAS.508.2946S/EPRINT_PDF)

**Citations:** 30

---

## Short Summary

We developed the first scalable, unsupervised GAN-based anomaly detector for million-galaxy survey data—identifying ~13,000 physically compelling outliers including rare mergers, tidal features, and extreme star-forming systems—and confirmed with follow-up spectroscopy that one such anomaly is a metal-poor dwarf galaxy hosting an unusually blue, higher-metallicity H II region, proving the method’s power to uncover genuinely new astrophysical phenomena.

**中文：** 我们开发了首个可扩展的、基于生成对抗网络（GAN）的无监督异常检测方法，用于百万星系巡天数据，成功识别出约13,000个具有明确物理意义的异常源，涵盖稀有并合星系、潮汐结构及极端恒星形成系统等；后续光谱观测证实，其中一例异常源为一个金属丰度极低的矮星系，其内部存在一个异常蓝、且金属丰度相对更高的H II区，有力证明了该方法在发现真正新颖天体物理现象方面的强大能力。

## Detailed Summary

This study addresses a critical and growing challenge in modern astronomy: the automated identification of rare, unexpected, or physically extreme astrophysical phenomena buried within petabyte-scale imaging surveys. As next-generation facilities like Rubin LSST prepare to deliver billions of galaxy images annually, traditional template-matching or supervised classification methods—reliant on pre-defined classes and labeled training data—are fundamentally ill-suited for discovering truly novel objects. Storey-Fisher et al. fill this gap with an innovative, fully unsupervised framework that does not assume prior knowledge of anomaly types, instead learning the intrinsic manifold of “normal” galaxy morphology directly from data. By targeting the Hyper Suprime-Cam (HSC) survey—a deep, high-resolution optical dataset covering over 1,000 deg²—the authors establish a scalable, physics-agnostic pipeline that prioritizes discovery potential over classification completeness, opening a new pathway for serendipitous science in the era of big-data astronomy.

The team trained a Wasserstein Generative Adversarial Network (WGAN) on nearly one million cutout images of galaxies drawn from HSC’s Wide layer, carefully curated to include only well-resolved, non-saturated, and non-stellar sources. Crucially, they leveraged both components of the GAN architecture for anomaly detection: the generator’s reconstruction error (quantifying how poorly a real image can be synthesized) and the discriminator’s learned feature-space embeddings (capturing subtle deviations from the dominant morphological distribution). Through systematic benchmarking, they demonstrated that the discriminator’s latent representations are significantly more sensitive to astrophysically meaningful outliers than either generator-based residuals or a conventional convolutional autoencoder—establishing a novel use case for adversarial networks in astronomy. To further refine interpretation, they introduced a creative two-stage characterization pipeline: first computing pixel-level residuals between real and WGAN-reconstructed images, then compressing those residuals via a dedicated convolutional autoencoder before applying UMAP for unsupervised clustering—enabling intuitive, low-dimensional exploration of anomaly subtypes without human labeling.

The method successfully isolated a high-confidence sample of ~13,000 anomalous galaxies—representing just ~1.3% of the input catalog but enriched by orders of magnitude in rare morphologies. Visual inspection and follow-up analysis revealed compelling candidates including advanced galaxy mergers with faint tidal streams, compact starbursts with extreme surface brightness, and systems exhibiting unusual asymmetries suggestive of recent interactions or feedback-driven disruption. Notably, spectroscopic validation of a single high-scoring anomaly confirmed it as a metal-poor dwarf galaxy hosting an exceptionally blue, higher-metallicity H II region—highlighting the technique’s ability to recover genuinely exotic systems missed by conventional selection criteria. The release of a publicly available anomaly score catalogue, open-source code, and an interactive web-based visualization platform (weirdgalaxi.es) ensures broad community utility and reproducibility. This work represents a conceptual and technical leap forward: it transforms generative modeling from a synthetic-data tool into a discovery engine, demonstrating that deep unsupervised learning can not only mirror astronomical data but actively illuminate its most informative outliers—thereby strengthening the foundation for AI-assisted exploration across current and future wide-field surveys.

### 中文版

本研究致力于解决现代天文学中一个日益突出且至关重要的挑战：在PB量级成像巡天数据中自动识别稀有、意外或物理性质极端的天体现象。随着Rubin LSST等新一代观测设施即将每年产出数十亿幅星系图像，依赖预定义类别与标注训练样本的传统模板匹配或监督分类方法，在本质上已难以胜任真正新奇天体的发现任务。Storey-Fisher等人填补了这一空白，提出了一种创新性的全无监督框架——该框架不预设任何异常类型先验知识，而是直接从数据中学习“正常”星系形态的内在流形结构。研究以Hyper Suprime-Cam（HSC）巡天——一项覆盖面积逾1000 deg²、具有高深度与高分辨率的光学成像数据集——为靶标，构建了一套可扩展、物理模型无关的分析流程；该流程以提升发现潜力为首要目标，而非追求分类完备性，从而为大数据时代的偶然性科学开辟了一条全新路径。

研究团队在HSC宽场巡天（Wide layer）中精心筛选出近百万幅星系切图图像进行训练，所有样本均满足良好分辨、未饱和且非恒星源等严格质量标准。尤为关键的是，作者充分利用了生成对抗网络（GAN）架构的双重组件实现异常检测：一方面利用生成器的重构误差（量化真实图像被合成的难易程度），另一方面则借助判别器所学习到的特征空间嵌入（捕捉偏离主导形态分布的细微偏差）。通过系统性基准测试，作者证实判别器的隐空间表征对具有天体物理意义的离群体敏感度显著优于仅基于生成器的残差方法，亦优于传统卷积自编码器——由此确立了对抗网络在天文学中一种全新的应用范式。为进一步增强可解释性，作者创新性地设计了两阶段表征流程：首先计算真实图像与WGAN重构图像之间的像素级残差，继而通过专用卷积自编码器对残差进行压缩，并最终采用UMAP进行无监督聚类——从而在无需人工标注的前提下，实现对异常子类直观、低维的探索。

该方法成功识别出约13,000个高置信度异常星系样本，仅占输入星系星表的约1.3%，却在稀有形态天体上实现了数量级级别的富集。目视检验与后续分析揭示了一批引人注目的候选体，包括伴有多条微弱潮汐尾的晚期并合星系、表面亮度极高的致密星暴星系，以及呈现异常不对称结构、暗示近期相互作用或反馈驱动扰动的特殊系统。尤为值得注意的是，对其中一例高分异常体开展的光谱证认确认其为一个贫金属矮星系，其内部存在一个异常蓝、且金属丰度更高的H II区——这凸显了该技术识别出真正奇异天体的能力，而此类天体往往被传统选源标准所遗漏。研究团队已公开发布异常评分星表、开源代码及交互式网络可视化平台（weirdgalaxi.es），确保成果在学界广泛可用且可复现。本工作代表了概念与技术层面的重大跃进：它将生成建模从一种合成数据工具，升华为一台真正的科学发现引擎；证明了深度无监督学习不仅能忠实复现天文数据，更能主动照亮其中最具信息量的离群体——从而为当前及未来各类宽视场巡天中的AI辅助探索奠定了更为坚实的基础。

## Key Figure

*To be added in future version.* See `2021_storeyfisher_anomaly_detection_hyper_suprimecam_galaxy_figures/` directory.
