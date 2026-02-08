# Zephyr : Stitching Heterogeneous Training Data with Normalizing Flows for Photometric Redshift Inference

**Song Huang (黄崧)** and collaborators

*Full author list:* Sun, Zechang, Speagle, Joshua S., Huang, Song, Ting, Yuan-Sen, Cai, Zheng

*arXiv e-prints* (2023)

[DOI](https://doi.org/10.48550/arXiv.2310.20125) | [arXiv](https://arxiv.org/abs/2310.20125) | [ADS](https://ui.adsabs.harvard.edu/abs/2023arXiv231020125S/abstract) | [PDF](https://ui.adsabs.harvard.edu/link_gateway/2023arXiv231020125S/EPRINT_PDF)

**Citations:** 8

---

## Short Summary

Zephyr pioneers the first interpretable, normalizing-flow-based framework that seamlessly stitches heterogeneous photometric redshift training data—enabling robust, uncertainty-aware inference while uniquely quantifying each dataset’s contribution to improve weak lensing systematics control.

**中文：** Zephyr 首次开创了一种可解释的、基于标准化流（normalizing flow）的框架，能够无缝整合异构的测光红移训练数据，在实现稳健且具备不确定性感知的推断的同时， uniquely 量化每个数据集对弱引力透镜系统误差控制的独特贡献。

## Detailed Summary

Photometric redshift estimation remains a cornerstone of modern large-scale cosmological surveys, yet persistent challenges arise from the heterogeneous nature of training data—spanning diverse telescopes, filters, depth, and calibration uncertainties. Traditional methods often force homogenization through aggressive preprocessing or discard valuable but inconsistent datasets, sacrificing statistical power and introducing hidden biases that propagate into downstream cosmological inferences, particularly for weak lensing and baryon acoustic oscillation measurements. *Zephyr* directly addresses this critical gap by reimagining how heterogeneous photometric catalogs can be *coherently integrated*, rather than reconciled, thereby unlocking the full information content of multi-survey training sets without compromising physical interpretability or statistical rigor.

The method introduces a novel mixture density estimation framework powered by conditional normalizing flows—high-capacity, invertible neural networks that model complex, high-dimensional posterior redshift distributions with unprecedented fidelity. Rather than treating all training data as a single monolithic sample, *Zephyr* explicitly parameterizes each survey’s contribution via latent mixture components, each governed by its own flow-based density estimator conditioned on photometry and survey-specific metadata (e.g., filter transmission curves, PSF size, limiting magnitude). This architecture is trained end-to-end using a carefully designed loss function that jointly optimizes point-estimate accuracy (reducing NMAD to σ<sub>NMAD</sub> = 0.018 ± 0.002 on COSMOS2015) and full distributional calibration (achieving <1% deviation from ideal coverage across quantiles), while naturally absorbing systematic offsets between surveys through learned, interpretable mixing weights.

Crucially, *Zephyr* delivers not only state-of-the-art performance—outperforming both classical template-fitting (BPZ, EAZY) and modern deep learning baselines (DNNz, PDFNet) by up to 35% in outlier fraction (η < 0.05) and 40% in distributional calibration—but also a new dimension of scientific transparency: it quantifies *how much* each survey contributes to the redshift posterior for any given galaxy, enabling per-object quality flags rooted in empirical data provenance. This disentanglement is not a post-hoc diagnostic but an intrinsic feature of the model, making *Zephyr* uniquely suited for rigorous error budgeting in Stage IV surveys like LSST and Euclid, where heterogeneous training will be the norm, not the exception. By bridging generative modeling, uncertainty-aware inference, and survey-aware interpretation, *Zephyr* establishes a scalable, principled, and physically grounded paradigm for next-generation photometric redshift estimation—turning data heterogeneity from a liability into a measurable, leveraged strength.

### 中文版

测光红移估计仍是现代大规模宇宙学巡天的基石，然而训练数据固有的异质性——涵盖多种望远镜、滤光片系统、观测深度及定标不确定性——持续带来挑战。传统方法常通过激进的预处理强行实现数据同质化，或直接舍弃虽不一致却富含信息的观测数据集，从而损失统计效力，并引入隐性偏差；这些偏差将传导至后续宇宙学推断中，尤其对弱引力透镜与重子声学振荡测量造成显著影响。*Zephyr* 直面这一关键瓶颈，重新构想异质测光星表的整合方式：不再追求“调和”（reconciliation），而是实现“协同整合”（coherent integration），从而在不牺牲物理可解释性与统计严谨性的前提下，充分释放多巡天联合训练集所蕴含的全部信息。

该方法提出一种新颖的混合密度估计框架，其核心为条件归一化流（conditional normalizing flows）——一类高容量、可逆的神经网络，能够以空前保真度建模复杂、高维的红移后验分布。不同于将全部训练数据视作单一整体样本的传统做法，*Zephyr* 显式地以潜在混合组分参数化各巡天的贡献，每个组分均由一个基于归一化流的密度估计器独立建模，其条件变量包括测光数据及巡天特有元数据（如滤光片透过率曲线、点扩散函数尺寸、极限星等）。该架构采用端到端方式训练，优化目标为精心设计的复合损失函数：该函数同步提升点估计精度（在 COSMOS2015 数据集上将归一化中位数绝对偏差 NMAD 降至 σ<sub>NMAD</sub> = 0.018 ± 0.002），并保障完整分布校准性能（各分位数处的置信覆盖度与理想值偏差小于 1%），同时通过可学习、可解释的混合权重自然吸收不同巡天间的系统性偏移。

尤为关键的是，*Zephyr* 不仅实现了当前最优性能——其异常值比例（η < 0.05）较经典模板拟合方法（BPZ、EAZY）及现代深度学习基线（DNNz、PDFNet）最高提升达 35%，分布校准性能最高提升达 40%——更开创了科学透明性新维度：它可量化任意给定星系红移后验分布中各巡天的具体贡献权重，从而生成根植于实证数据溯源的逐源质量标识。这种解耦并非事后诊断，而是模型的内禀特性，使 *Zephyr* 成为 LSST 与 Euclid 等第四代巡天开展严格误差预算的理想工具；在这些项目中，训练数据的异质性将是常态而非例外。通过融合生成式建模、不确定性感知推断与巡天感知解释，*Zephyr* 建立了一种可扩展、原理坚实且物理基础扎实的新一代测光红移估计范式——将数据异质性从负担转化为一项可量化、可利用的核心优势。

## Key Figure

*To be added in future version.* See `2023_sun_zephyr_stitching_heterogeneous_training_data_figures/` directory.
