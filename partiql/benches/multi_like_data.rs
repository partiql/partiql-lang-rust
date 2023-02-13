use itertools::Itertools;
use partiql_eval::env::basic::MapBindings;
use partiql_value::{partiql_tuple, Bag, Tuple, Value};
use rand::{Rng, SeedableRng};

pub const QUERY_1: &str = "
            SELECT *
            FROM hr.employees as emp
            WHERE lower(emp.name) LIKE '%bob smith%'
            ";

pub const QUERY_15: &str = "
            SELECT *
            FROM hr.employees as emp
            WHERE lower(emp.name) LIKE '%bob smith%'
               OR lower(emp.name) LIKE '%gage swanson%'
               OR lower(emp.name) LIKE '%riley perry%'
               OR lower(emp.name) LIKE '%sandra woodward%'
               OR lower(emp.name) LIKE '%abagail oconnell%'
               OR lower(emp.name) LIKE '%amari duke%'
               OR lower(emp.name) LIKE '%elisha wyatt%'
               OR lower(emp.name) LIKE '%aryanna hess%'
               OR lower(emp.name) LIKE '%bryanna jones%'
               OR lower(emp.name) LIKE '%trace gilmore%'
               OR lower(emp.name) LIKE '%antwan stevenson%'
               OR lower(emp.name) LIKE '%julianna callahan%'
               OR lower(emp.name) LIKE '%jaelynn trevino%'
               OR lower(emp.name) LIKE '%kadence bates%'
               OR lower(emp.name) LIKE '%jakobe townsend%'
            ";

pub const QUERY_30: &str = "
            SELECT *
            FROM hr.employees as emp
            WHERE lower(emp.name) LIKE '%bob smith%'
               OR lower(emp.name) LIKE '%gage swanson%'
               OR lower(emp.name) LIKE '%riley perry%'
               OR lower(emp.name) LIKE '%sandra woodward%'
               OR lower(emp.name) LIKE '%abagail oconnell%'
               OR lower(emp.name) LIKE '%amari duke%'
               OR lower(emp.name) LIKE '%elisha wyatt%'
               OR lower(emp.name) LIKE '%aryanna hess%'
               OR lower(emp.name) LIKE '%bryanna jones%'
               OR lower(emp.name) LIKE '%trace gilmore%'
               OR lower(emp.name) LIKE '%antwan stevenson%'
               OR lower(emp.name) LIKE '%julianna callahan%'
               OR lower(emp.name) LIKE '%jaelynn trevino%'
               OR lower(emp.name) LIKE '%kadence bates%'
               OR lower(emp.name) LIKE '%jakobe townsend%'
               OR lower(emp.name) LIKE '%austin pennington%'
               OR lower(emp.name) LIKE '%colby woodward%'
               OR lower(emp.name) LIKE '%brycen blair%'
               OR lower(emp.name) LIKE '%cristal mercer%'
               OR lower(emp.name) LIKE '%river gilmore%'
               OR lower(emp.name) LIKE '%saniya bowers%'
               OR lower(emp.name) LIKE '%braedon ross%'
               OR lower(emp.name) LIKE '%clark mejia%'
               OR lower(emp.name) LIKE '%ryan day%'
               OR lower(emp.name) LIKE '%marilyn luna%'
               OR lower(emp.name) LIKE '%avah sanchez%'
               OR lower(emp.name) LIKE '%amelie wu%'
               OR lower(emp.name) LIKE '%paola duke%'
               OR lower(emp.name) LIKE '%jesse trevino%'
               OR lower(emp.name) LIKE '%bianca cisneros%'
            ";

/// Return a sequence of 10201 `Value`s where each is a `Tuple` of the form
/// `{id: <num>, name: "<random prefix> <name1> <name2> <random suffix>"}`
pub fn employees() -> Vec<Value> {
    let name1 = vec![
        "Bob",
        "Madden",
        "Brycen",
        "Bryanna",
        "Zayne",
        "Jocelynn",
        "Breanna",
        "Margaret",
        "Jasmine",
        "Kenyon",
        "Aryanna",
        "Zackery",
        "Jorden",
        "Malia",
        "Raven",
        "Neveah",
        "Finley",
        "Austin",
        "Jaxson",
        "Tobias",
        "Dominique",
        "Devan",
        "Colby",
        "Tanner",
        "Mckenna",
        "Kristina",
        "Cristal",
        "River",
        "Taliyah",
        "Abagail",
        "Spencer",
        "Gage",
        "Ronnie",
        "Amari",
        "Jabari",
        "Alanna",
        "Anderson",
        "Saniya",
        "Baylee",
        "Elisa",
        "Savannah",
        "Jakobe",
        "Sandra",
        "Simone",
        "Frank",
        "Braedon",
        "Clark",
        "Francisco",
        "Roman",
        "Matias",
        "Messi",
        "Elisha",
        "Alexander",
        "Kadence",
        "Karsyn",
        "Adonis",
        "Ishaan",
        "Trevon",
        "Ryan",
        "Jaelynn",
        "Marilyn",
        "Emma",
        "Avah",
        "Jordan",
        "Riley",
        "Amelie",
        "Denisse",
        "Darion",
        "Lydia",
        "Marley",
        "Brogan",
        "Trace",
        "Maeve",
        "Elijah",
        "Kareem",
        "Erick",
        "Hope",
        "Elisabeth",
        "Antwan",
        "Francesca",
        "Layla",
        "Jase",
        "Angel",
        "Addyson",
        "Mckinley",
        "Julianna",
        "Winston",
        "Royce",
        "Paola",
        "Issac",
        "Zachary",
        "Niko",
        "Shania",
        "Colin",
        "Jesse",
        "Pedro",
        "Cheyenne",
        "Ashley",
        "Karli",
        "Bianca",
        "Mario",
    ];
    let name2 = vec![
        "Smith",
        "Oconnell",
        "Whitehead",
        "Carrillo",
        "Parrish",
        "Monroe",
        "Summers",
        "Hurst",
        "Durham",
        "Hardin",
        "Hunt",
        "Mitchell",
        "Pennington",
        "Woodward",
        "Franklin",
        "Martinez",
        "Shepard",
        "Khan",
        "Mcfarland",
        "Frey",
        "Mckenzie",
        "Blair",
        "Mercer",
        "Callahan",
        "Cameron",
        "Gilmore",
        "Bowers",
        "Donovan",
        "Meyers",
        "Horne",
        "Rice",
        "Castillo",
        "Cain",
        "Dickson",
        "Valenzuela",
        "Silva",
        "Prince",
        "Vance",
        "Berry",
        "Coffey",
        "Young",
        "Walker",
        "Burch",
        "Ross",
        "Mejia",
        "Zuniga",
        "Haney",
        "Jordan",
        "Love",
        "Larsen",
        "Bowman",
        "Werner",
        "Greer",
        "Krause",
        "Bishop",
        "Day",
        "Luna",
        "Patrick",
        "Adkins",
        "Benson",
        "Mcconnell",
        "Sanchez",
        "Villa",
        "Wu",
        "Duke",
        "Fisher",
        "Hess",
        "Lawrence",
        "Perry",
        "Hardy",
        "Wyatt",
        "Mcknight",
        "Thomas",
        "Trevino",
        "Flowers",
        "Cisneros",
        "Coleman",
        "Sanders",
        "Good",
        "Newton",
        "Carpenter",
        "Garza",
        "Barber",
        "Swanson",
        "Owen",
        "Anderson",
        "Bright",
        "Beck",
        "Lawson",
        "Jones",
        "Davila",
        "Porter",
        "Dougherty",
        "Stevenson",
        "Malone",
        "Garrison",
        "Bates",
        "Wheeler",
        "Petty",
        "Rojas",
        "Townsend",
    ];

    // cartesian product of name1 x name2 (e.g., "Bob Smith", ... "Mario Townsend")
    let combined = name1
        .iter()
        .cartesian_product(name2.iter())
        .map(|(n1, n2)| format!("{n1} {n2}"));

    // seed the rng with a known value to assure same data across runs
    let mut rng = rand::rngs::StdRng::from_seed([42; 32]);
    use rand::distributions::Distribution;
    let chars = rand::distributions::Alphanumeric;
    let random_size = rand::distributions::uniform::Uniform::from(5..=100);

    // add random string prefix and suffix to each combined name
    let employee_data: Vec<Value> = combined
        .enumerate()
        .map(|(id, person)| {
            let prefix_size = random_size.sample(&mut rng);
            let suffix_size = random_size.sample(&mut rng);
            let prefix: String = (0..prefix_size)
                .map(|_| rng.sample(chars) as char)
                .collect();
            let suffix: String = (0..suffix_size)
                .map(|_| rng.sample(chars) as char)
                .collect();
            let full_name = format!("{prefix} {person} {suffix}");
            partiql_tuple![("id", id), ("name", full_name)].into()
        })
        .collect_vec();

    employee_data
}

pub fn employee_data() -> MapBindings<Value> {
    let data = partiql_tuple![("hr", partiql_tuple![("employees", Bag::from(employees()))])];

    data.into()
}
