# Knowledge Graph in Astronomical Research with Large Language Models: Quantifying Driving Forces in Interdisciplinary Scientific Discovery

**Song Huang (黄崧)** and collaborators

*Full author list:* Sun, Zechang, Ting, Yuan-Sen, Liang, Yaobo, Duan, Nan, Huang, Song, Cai, Zheng

*arXiv e-prints* (2024)

[DOI](https://doi.org/10.48550/arXiv.2406.01391) | [arXiv](https://arxiv.org/abs/2406.01391) | [ADS](https://ui.adsabs.harvard.edu/abs/2024arXiv240601391S/abstract) | [PDF](https://ui.adsabs.harvard.edu/link_gateway/2024arXiv240601391S/EPRINT_PDF)

**Citations:** 11

---

## Short Summary

We built the first large-scale astronomical knowledge graph—spanning 31 years and 298,000 papers—to objectively quantify how numerical simulations and machine learning reshape discovery, revealing that while AI has deeply penetrated astronomy, the field now faces a critical bottleneck: a striking absence of *new, hybrid concepts* at the AI–astronomy interface, signaling where transformative innovation must now be cultivated.

**中文：** 我们构建了首个大规模天文知识图谱——涵盖31年时间跨度、298,000篇论文——以客观量化数值模拟与机器学习如何重塑天文发现。结果表明：尽管人工智能已深度融入天文学，该领域当前却面临一个关键瓶颈：在人工智能与天文学交叉界面处，显著缺乏*全新、融合性概念*，这正昭示着亟需培育突破性创新的方向。

## Detailed Summary

This study addresses a critical and long-standing gap in our understanding of how interdisciplinary innovation actually unfolds in astronomy—moving beyond anecdotal accounts or coarse bibliometric proxies to quantitatively trace *how* new technologies reshape scientific thinking over time. While the adoption of tools like numerical simulations and machine learning in astronomy has been widely documented, no prior work has systematically measured *when*, *how deeply*, and *in what conceptual contexts* these technologies become integrated into the discipline’s intellectual fabric. By treating astronomical knowledge not as static keywords but as evolving, relational concepts embedded in scholarly discourse, this research pioneers a dynamic, fine-grained framework for studying scientific transformation—one that captures the subtle, cumulative process by which external innovations seed new questions, methods, and subfields.

The team developed an innovative hybrid methodology combining large language models (LLMs) with temporal network science to construct the first high-resolution, concept-level knowledge graph of modern astronomy. From a corpus of 297,807 peer-reviewed publications spanning thirty-one years (1993–2024), they employed carefully fine-tuned LLMs—not merely for keyword extraction, but for contextual concept identification and disambiguation—yielding a curated ontology of 24,939 distinct, semantically grounded concepts (e.g., “adaptive optics,” “cosmic microwave background lensing,” “graph neural networks for galaxy classification”). Crucially, link strengths in the knowledge graph were derived from citation-reference co-occurrence patterns across rolling five-year windows, enabling robust, time-resolved measurement of conceptual relevance and cross-pollination. This approach goes far beyond traditional co-citation or keyword co-occurrence analyses by capturing *semantic proximity* as expressed through scholarly usage and argumentative framing.

The analysis reveals two distinct, empirically identifiable phases in the integration of transformative technologies: an initial *assimilation phase*, where simulation and ML concepts rapidly accumulate strong links to established astronomical domains (e.g., “N-body simulations” linked to “dark matter halos” surged in relevance between 2005–2010; “convolutional neural networks” gained strongest ties to “transient detection” between 2016–2021), followed by an *exploration phase*, where those technologies begin generating novel, domain-specific variants (e.g., “hydrodynamical simulations with radiative transfer” or “physics-informed neural networks for stellar atmospheres”). Most strikingly, the graph uncovers a pronounced conceptual stagnation at the AI–astronomy interface: while ML-related concepts now appear in over 38% of all astronomy papers published since 2022, fewer than 0.7% of newly emerging concepts (those first appearing after 2020) reside *at the intersection* of AI and core astrophysical theory or observation—suggesting that current applications remain largely instrumental rather than generative. This bottleneck is not one of adoption, but of *co-creation*: the field has yet to cultivate a shared conceptual vocabulary that bridges algorithmic design and astrophysical insight. These findings provide both a diagnostic tool and a strategic compass—highlighting where targeted investment in cross-disciplinary training, collaborative infrastructure, and theory-aware AI development can catalyze the next wave of discovery.

### 中文版

本研究致力于解决天文学领域一个长期存在且至关重要的认知空白：即跨学科创新在现实中究竟如何发生——超越零散的个案叙述或粗粒度的文献计量代理指标，转而定量追踪新技术如何随时间推移重塑科学思维。尽管数值模拟与机器学习等工具在天文学中的应用已广为人知，但此前尚无任何研究系统性地测量这些技术究竟在*何时*、以*何种深度*、以及在*哪些概念语境*中真正融入了该学科的知识肌理。本研究摒弃将天文知识视为静态关键词的传统范式，转而将其建模为嵌入学术话语之中的、动态演化的、具有关系结构的概念体系，从而开创了一种动态、细粒度的科学变革研究框架——该框架能够精准刻画外部创新如何以微妙而累积的方式催生新问题、新方法乃至新子学科。

研究团队开发了一种创新性的混合方法学，将大语言模型（LLMs）与时间网络科学相结合，构建出首张高分辨率、概念层级的现代天文学知识图谱。研究基于涵盖31年（1993–2024年）的297,807篇同行评议论文构成的语料库，采用精心微调的LLMs，不仅实现关键词抽取，更聚焦于上下文敏感的概念识别与歧义消解，最终构建出一个包含24,939个语义明确、彼此区分的概念本体（例如：“自适应光学”、“宇宙微波背景弱引力透镜效应”、“用于星系分类的图神经网络”）。尤为关键的是，知识图谱中各概念间的连接强度，源自滚动五年时间窗口内引文—参考文献共现模式的统计分析，从而实现了对概念相关性与跨领域渗透效应稳健、时序分辨的量化测量。该方法远超传统共引分析或关键词共现分析，其核心在于捕捉学者在实际论述与论证框架中所体现的*语义邻近性*。

分析结果揭示了颠覆性技术融入天文学过程中的两个经验可辨识的阶段：首先是*同化阶段*，即模拟与机器学习相关概念迅速与既有天文领域建立强关联（例如，“N体模拟”与“暗物质晕”的关联强度在2005–2010年间显著跃升；“卷积神经网络”与“暂现源探测”的最强关联则出现在2016–2021年间）；随后进入*探索阶段*，即此类技术开始衍生出新颖的、面向特定天文领域的变体（例如：“含辐射转移的流体动力学模拟”或“面向恒星大气建模的物理信息神经网络”）。最引人注目的是，图谱清晰揭示出人工智能与天文学交叉界面存在显著的概念停滞现象：尽管2022年以来，与机器学习相关的概念已出现在逾38%的天文学论文中，但2020年后首次出现的新概念中，仅有不足0.7%真正位于人工智能与核心天体物理理论或观测的*交叉地带*——这表明当前应用仍主要停留在工具性层面，尚未展现出生成性潜力。这一瓶颈并非源于技术采纳不足，而在于*协同创造*的缺失：学界尚未培育出一套贯通算法设计与天体物理洞见的共享概念语言。上述发现既提供了一种诊断性工具，亦指明了一条战略路径——凸显出在跨学科人才培养、协作基础设施建设以及理论驱动型人工智能研发等方面进行定向投入，将如何切实催化下一轮重大科学发现。

## Key Figure

*To be added in future version.* See `2024_sun_knowledge_graph_astronomical_research_large_figures/` directory.
