use egui_plot::{Line, Plot, PlotPoints, Points, VLine};
use rand::Rng;

use crate::algorithm::{Agent, GenerationData, Optimum};

type XFunction = fn(f64) -> f64;

const FUNCTIONS: [(XFunction, &str); 3] = [
    (|x: f64| x.sin() + x / 3.0, "sin(x) + x/3"),
    (|x: f64| x.cos() + x / 3.0, "cos(x) + x/3"),
    (|x: f64| 3.0 * x * x.sin() + x / 3.0, "3*x*sin(x) + x/3"),
];

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct LabApp {
    a: f64,
    b: f64,
    /// Плюс к отображению
    display_adjustment: f64,
    /// Ищем максимумы или минимумы
    optimum: Optimum,
    /// Сколько будет создано популяций
    populations_count: usize,
    /// На какое значение в + или – мутирует агент
    mutation_intensity: f64,
    /// Вероятность мутации
    mutation_probability: f64,
    /// Количество агентов в поколении
    population_size: usize,
    #[serde(skip)]
    function: (XFunction, &'static str),
    #[serde(skip)]
    simulated_populations: Option<Vec<GenerationData>>,
    #[serde(skip)]
    current_generation: usize,
}

impl Default for LabApp {
    fn default() -> Self {
        Self {
            a: 1.0,
            b: 2.0,
            display_adjustment: 0.0,
            optimum: Optimum::Minimum,
            populations_count: 10,
            mutation_intensity: 0.5,
            mutation_probability: 0.1,
            population_size: 50,
            function: FUNCTIONS[0],
            current_generation: 0,
            simulated_populations: None,
        }
    }
}

impl LabApp {
    /// Запускается в самом начале
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_theme(egui::Theme::Light);

        // Загрузка предыдущего состояния приложения в случае если оно существует
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    /// Запуск генетического алгоритма
    fn run_simulation(&self) -> Vec<GenerationData> {
        let mut rng = rand::rng();
        let fun = self.function.0;
        let mut history: Vec<GenerationData> = Vec::with_capacity(self.populations_count);
        // Инициализация начальной популяции
        let mut population: Vec<Agent> = (0..self.population_size)
            .map(|_| {
                let x = rng.random_range(self.a..=self.b);
                let mut agent = Agent::new(x);
                agent.calculate(fun);
                agent
            })
            .collect();

        for _ in 0..self.populations_count {
            population.sort_by(|a, b| match self.optimum {
                Optimum::Maximum => b.y.partial_cmp(&a.y).unwrap(),
                Optimum::Minimum => a.y.partial_cmp(&b.y).unwrap(),
            });
            let survivors = population[..population.len() / 2].to_vec();
            let discarded = population[population.len() / 2..].to_vec();
            history.push(GenerationData {
                survivors: survivors.clone(),
                discarded: discarded.clone(),
            });

            // Репродукция: выжившие остаются, потомки (как среднее значение родителей) добавляются до восстановления полной популяции
            let mut new_population = survivors.clone();
            while new_population.len() < self.population_size {
                let parent1 = survivors[rng.random_range(0..survivors.len())].clone();
                let parent2 = survivors[rng.random_range(0..survivors.len())].clone();
                let child_x = (parent1.x + parent2.x) / 2.0;
                let mut child = Agent::new(child_x);
                child.calculate(fun);
                new_population.push(child);
            }
            for agent in new_population.iter_mut() {
                agent.mutate(self.mutation_intensity, self.mutation_probability);
                agent.calculate(fun);
            }
            population = new_population;
        }
        history
    }
}

impl eframe::App for LabApp {
    /// Сохранение данных при выходе
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Отрисовка графического интерфейса
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Разработка генетического алгоритма поиска экстремума функции");

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("A:");
                        ui.add(egui::DragValue::new(&mut self.a).speed(0.1));
                    });
                    ui.horizontal(|ui| {
                        ui.label("B:");
                        ui.add(egui::DragValue::new(&mut self.b).speed(0.1));
                    });
                    ui.horizontal(|ui| {
                        ui.label("D:");
                        ui.add(egui::DragValue::new(&mut self.display_adjustment).speed(0.1));
                    });
                });
                ui.vertical(|ui| {
                    ui.label("Оптимум:");
                    egui::ComboBox::from_id_salt("Оптимум")
                        .selected_text(match self.optimum {
                            Optimum::Minimum => "Минимум",
                            Optimum::Maximum => "Максимум",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.optimum, Optimum::Minimum, "Минимум");
                            ui.selectable_value(&mut self.optimum, Optimum::Maximum, "Максимум");
                        });
                });
                ui.vertical(|ui| {
                    ui.label("Кол-во популяций:");
                    ui.add(egui::DragValue::new(&mut self.populations_count).speed(1));
                });
                ui.vertical(|ui| {
                    ui.label("Интенсивность мутации:");
                    ui.add(
                        egui::DragValue::new(&mut self.mutation_intensity)
                            .range(-1.0..=1.0)
                            .speed(0.1),
                    );
                    ui.label("Функция:");
                    egui::ComboBox::from_id_salt("Функция")
                        .selected_text(self.function.1)
                        .show_ui(ui, |ui| {
                            for function in FUNCTIONS {
                                ui.selectable_value(&mut self.function, function, function.1);
                            }
                        });
                });
                ui.vertical(|ui| {
                    ui.label("Частота мутации:");
                    ui.add(
                        egui::DragValue::new(&mut self.mutation_probability)
                            .range(0.0..=1.0)
                            .speed(0.1),
                    );
                });
                ui.vertical(|ui| {
                    ui.label("Количество особей:");
                    ui.add(egui::DragValue::new(&mut self.population_size).speed(1));
                });

                if ui.button("Старт").clicked() {
                    self.simulated_populations = Some(self.run_simulation());
                    self.current_generation = 0;
                }

                if let Some(history) = &self.simulated_populations {
                    ui.add(
                        egui::Slider::new(&mut self.current_generation, 0..=history.len() - 1)
                            .text("Поколение"),
                    );
                }
            });

            Plot::new("plot").show(ui, |plot_ui| {
                // Отрисовка функции
                let x_range =
                    (self.a - self.display_adjustment)..(self.b + self.display_adjustment);
                let line = Line::new(PlotPoints::from_explicit_callback(
                    self.function.0,
                    x_range,
                    100,
                ))
                .name(self.function.1);
                plot_ui.line(line);

                // Вертикальные линии
                plot_ui.vline(VLine::new(self.a).name("A"));
                plot_ui.vline(VLine::new(self.b).name("B"));

                if let Some(history) = &self.simulated_populations {
                    if self.current_generation < history.len() {
                        let generation = &history[self.current_generation];
                        let discarded_points: Vec<[f64; 2]> = generation
                            .discarded
                            .iter()
                            .map(|agent| [agent.x, agent.y.unwrap_or(0.0)])
                            .collect();
                        let survivors_points: Vec<[f64; 2]> = generation
                            .survivors
                            .iter()
                            .map(|agent| [agent.x, agent.y.unwrap_or(0.0)])
                            .collect();
                        plot_ui.points(
                            Points::new(PlotPoints::new(discarded_points))
                                .color(egui::Color32::RED)
                                .radius(3.0)
                                .name("Худшие особи"),
                        );
                        plot_ui.points(
                            Points::new(PlotPoints::new(survivors_points))
                                .color(egui::Color32::GREEN)
                                .radius(3.0)
                                .name("Лучшие особи"),
                        );
                    }
                }
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.label("Создано Минкиным Александром, группа 221-373");
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
