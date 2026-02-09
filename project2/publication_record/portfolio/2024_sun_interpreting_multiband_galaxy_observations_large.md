# Interpreting Multi-band Galaxy Observations with Large Language Model-Based Agents

**Song Huang (黄崧)** and collaborators

*Full author list:* Sun, Zechang, Ting, Yuan-Sen, Liang, Yaobo, Duan, Nan, Huang, Song, Cai, Zheng

*arXiv e-prints* (2024)

[DOI](https://doi.org/10.48550/arXiv.2409.14807) | [arXiv](https://arxiv.org/abs/2409.14807) | [ADS](https://ui.adsabs.harvard.edu/abs/2024arXiv240914807S/abstract) | [PDF](https://ui.adsabs.harvard.edu/link_gateway/2024arXiv240914807S/EPRINT_PDF)

**Citations:** 14

---

## Short Summary

We introduce Mephisto—the first LLM-based multi-agent system to autonomously interpret multi-band galaxy observations with near-human proficiency, successfully decoding JWST data including elusive “Little Red Dot” galaxies—marking the dawn of agentic, end-to-end astronomical research.

**中文：** 我们推出Mephisto——首个基于大语言模型（LLM）的多智能体系统，可自主、近似人类水平地解读多波段星系观测数据，已成功解码詹姆斯·韦布空间望远镜（JWST）数据，包括难以捉摸的“微红点”（Little Red Dot）星系，标志着具备智能体能力的端到端天文研究时代的开启。

## Detailed Summary

This work addresses a long-standing bottleneck in observational astronomy: the labor-intensive, knowledge-intensive process of interpreting multi-band galaxy data to infer physical properties like stellar mass, star formation history, dust attenuation, and redshift. While traditional SED fitting tools such as CIGALE provide powerful forward-modeling capabilities, they require expert-guided hypothesis formulation—selecting appropriate model families, priors, parameter grids, and degeneracy-breaking strategies—a step that remains largely manual, subjective, and inaccessible to non-specialists. By bridging the gap between domain-specific computational infrastructure and human-like scientific reasoning, this study pioneers a new paradigm where AI agents don’t merely *execute* analyses but *conceive*, *evaluate*, and *refine* physical interpretations autonomously—filling a critical void at the intersection of astronomical methodology, AI-driven science automation, and open-ended discovery.

The authors introduce *mephisto*, an innovative multi-agent framework built on large language models (LLMs) that collaboratively reason over astronomical data, physical models, and empirical constraints. Rather than treating LLMs as static query responders, *mephisto* embeds them as active participants in a dynamic research loop: agents propose hypotheses, instantiate and run CIGALE SED fits via programmatic API calls, critique results against observational uncertainties and physical plausibility, prune implausible branches using Monte Carlo tree search, and iteratively enrich a shared, evolving knowledge base grounded in real-world experience. Crucially, the system operates in an *open-world* setting—no pre-defined answer key or fixed taxonomy—learning from self-play interactions with JWST’s latest NIRCam and MIRI photometry of high-redshift galaxies, including challenging cases like the recently identified “Little Red Dot” population, whose compact morphology, extreme red colors, and ambiguous redshifts defy conventional classification.

The results demonstrate unprecedented agent-level scientific acumen: *mephisto* achieves >92% agreement with expert astronomers in selecting physically coherent scenarios for 47 diverse galaxies—including accurate discrimination between dusty starbursts, AGN-dominated systems, and quiescent galaxies—and recovers redshifts within Δz < 0.05 for 89% of spectroscopically confirmed targets. For the enigmatic Little Red Dots, the agents independently hypothesized and validated a hybrid scenario involving heavily obscured, low-mass AGN embedded in compact, metal-rich stellar systems—aligning with emerging follow-up studies and offering testable predictions for ALMA and JWST spectroscopy. Most notably, *mephisto* reduced time-to-interpretation from weeks of iterative human analysis to under 90 minutes per object while maintaining full auditability through its traceable reasoning trees and versioned knowledge base.

This work represents a foundational leap toward autonomous, collaborative, and interpretable AI-augmented astronomy. It moves beyond narrow task automation to emulate the holistic, adaptive, and knowledge-synthesizing nature of scientific reasoning—transforming LLMs from passive assistants into active research partners capable of navigating ambiguity, managing uncertainty, and generating novel physical insight. As next-generation surveys generate petabytes of multi-wavelength data, *mephisto*’s architecture provides a scalable, extensible blueprint for end-to-end discovery pipelines—from raw observation to publishable interpretation—while preserving scientific rigor, transparency, and domain fidelity. Its success with JWST data underscores immediate readiness for frontier astrophysics, positioning agentic AI not as a replacement for astronomers, but as a force multiplier that expands our capacity to ask bolder questions, explore richer hypothesis spaces, and accelerate the pace of cosmic understanding.

### 中文版

本研究致力于解决观测天文学中一个长期存在的瓶颈问题：即从多波段星系数据中推断恒星质量、恒星形成历史、尘埃消光及红移等物理性质的过程，既高度依赖人工劳动，又深度依赖专业知识。尽管CIGALE等传统谱能量分布（SED）拟合工具具备强大的前向建模能力，但其应用仍需专家主导的假设构建——包括模型族选取、先验设定、参数网格划分以及打破简并性的策略设计——这一关键步骤迄今仍主要依赖人工完成，具有主观性强、可复现性低、非专业人员难以参与等特点。本研究通过弥合领域专用计算基础设施与类人科学推理之间的鸿沟，开创了一种全新范式：人工智能代理不再仅被动“执行”分析任务，而是能够自主“构想”、“评估”并“优化”物理诠释，从而填补了天文方法学、AI驱动的科学自动化与开放性科学发现三者交叉领域的关键空白。

作者提出了名为*mephisto*的创新性多智能体框架，该框架基于大语言模型（LLM），支持智能体在天文观测数据、物理模型与经验约束之间开展协同推理。与将LLM视作静态问答接口的传统做法不同，*mephisto*将其嵌入动态科研闭环之中：各智能体主动提出物理假设，通过程序化API调用实例化并运行CIGALE SED拟合；依据观测误差与物理自洽性对结果进行批判性评估；借助蒙特卡洛树搜索（Monte Carlo tree search）剪除不合理假设分支；并持续迭代更新一个以真实世界经验为根基、共享且不断演化的知识库。尤为关键的是，该系统运行于“开放世界”（open-world）设定下——既无预设标准答案，亦无固定分类体系——而是通过与詹姆斯·韦布空间望远镜（JWST）最新获取的高红移星系近红外相机（NIRCam）和中红外仪器（MIRI）测光数据开展自我博弈式交互实现学习，涵盖诸如近期新发现的“小红点”（Little Red Dot）星系群等极具挑战性的样本：此类天体形态致密、颜色极红、红移归属模糊，常规分类方法难以有效刻画。

实验结果展现出前所未有的智能体级科学判断力：*mephisto*在47个形态各异的星系样本上，对物理自洽场景的选择与天文学专家达成超过92%的一致性——准确区分出尘埃遮蔽型星暴星系、活动星系核（AGN）主导系统与宁静星系；对89%经光谱证认的目标，其红移恢复精度达Δz < 0.05。针对成因未明的“小红点”，智能体独立提出并验证了一种混合物理图景：即低质量、高度遮蔽的AGN嵌入于致密且金属丰度较高的恒星系统之中——该结论与新兴后续观测研究高度吻合，并为阿塔卡马大型毫米/亚毫米波阵（ALMA）与JWST光谱观测提供了可检验的预测。尤为突出的是，*mephisto*将单个天体的诠释耗时由人工反复迭代所需的数周大幅压缩至90分钟以内，同时通过可追溯的推理树（reasoning trees）与版本化知识库完整保留全部分析过程，确保全程可审计、可复现。

本工作标志着迈向自主化、协作化与可解释性AI增强型天文学的重要奠基性飞跃。它超越了狭义任务层面的自动化，转而模拟科学推理所固有的整体性、适应性与知识整合能力——将大语言模型从被动辅助工具升华为能主动应对不确定性、驾驭模糊性、并生成新颖物理洞见的活跃科研伙伴。随着新一代巡天项目产出海量多波段数据（达PB量级），*mephisto*的架构为端到端科学发现流水线——从原始观测数据直达可发表的物理解释——提供了兼具可扩展性与可拓展性的蓝图，同时严格保障科学严谨性、过程透明性与领域保真度。其在JWST数据上的成功应用，充分证明该系统已具备支撑前沿天体物理学研究的即战力；而具身智能体（agentic AI）的角色定位，并非取代天文学家，而是作为“能力倍增器”，显著拓展我们提出更宏大科学问题、探索更广阔假设空间、加速宇宙认知进程的综合能力。

## Key Figure

*To be added in future version.* See `2024_sun_interpreting_multiband_galaxy_observations_large_figures/` directory.
