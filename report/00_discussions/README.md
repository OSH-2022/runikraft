# 小组讨论摘要

## 2022-03-30

- 讨论目标：结束调研报告的撰写，开始可行性报告的撰写
- 讨论过程：
    - 确定许可证；
    - 审阅调研报告；
    - 交流Rust学习进度；
    - 讨论可行性报告分工；

- 讨论结果
    - 程序使用 BSD 3-Clause 许可证，报告使用 CC-BY-4.0 许可证；
    - 确认调研报告不需要继续修改；
    - 可行性报告分工：
        - 理论依据：郭耸霄
        - 技术依据 （主要使用的工具：`rustc`、`cargo`、`qemu`）
            - Rust能写操作系统：陈建绿
            - RISC-V的特权指令，KVM能支持RISC-V架构：吴骏东

        - 创新点+概要设计：张子辰、蓝俊玮

    - 可行性报告初稿的完成时间是2022-04-05


## 2022-03-19

- 讨论目标：确定时间安排
- 讨论过程：
  - 最终确定方向；
  - 更改小组名称；
  - 讨论时间安排；
  - 讨论调研报告分工；
- 讨论结果：
  - 方向确定为Rust实现Unikernel；
  - 小组名称确定为runikraft；
  - 争取2022-04-05 前学习完Rust语言、及完成可行性报告；
  - 争取2022-03-25 前完成调研报告；
  - 筛选出rumprun、rusty-hermit、mirageOS、includeOS、clickOS 5个未过时的项目；
  - 郭耸霄负责调研rust优越，兼容性的重要性；
  - 张子辰负责调研Unikraft；
  - 蓝俊玮负责调研clickOS、includeOS、miageOS；
  - 陈建绿负责调研rusty-hermit、rumprun；
  - 吴骏东负责调研往年有关Unikernel的项目；
  - 将每周三第五节确定为讨论时间。

## 2022-03-12

- 讨论目标：听取老师建议并确定选题。
- 讨论过程：
  - 询问老师参会方式；
  - 得知老师采取腾讯会议参会后，登入腾讯会议并等待；
  - 在老师长时间未进入讨论后，关闭腾讯会议，将讨论模式改为分享调研成果；
  - 蓝俊玮分享了安全容器调研；
  - 吴骏东分享了人机接口调研；
  - 张子辰分享了Unikernel调研；
  - 郭耸霄分享了树莓派上实现操作系统调研；
  - 陈建绿分享了物联网操作系统调研；
  - 张子辰分享了智能电梯系统构想。
- 讨论结果：
  - 否定了上次讨论中确定的首选方向和备选方向；
  - 为进一步的实验准备，应该开始学习Rust语言；
  - 主要研究方向集中于Rust、树莓派、人机交互3个关键词；
  - 进一步调研，为下一次与老师的讨论汇报做好准备。

## 2022-03-05

- 讨论目标：确定初步感兴趣的大作业题目。
- 讨论过程：
  - 确定组名为LJW组；
  - 建立 Github 项目x-LJW；
  - 浏览往届大作业项目；
  - 通过排除法确定方向。
- 讨论结果：
  - 首选方向：分布式文件系统；
  - 备选方向：图形化界面的 Unikernel；