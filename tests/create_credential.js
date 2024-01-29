import http from "k6/http"
import { check } from 'k6';
export const options = {
    iterations: 1,
    duration: '5m',
    thresholds: {
        http_req_failed: ['rate<0.01'], // http errors should be less than 1%
        http_req_duration: ['p(95)<500'], // 95 percent of response times must be below 500ms
    },
};
export default function() {
    const URL = __ENV.GRAPHQL_CORE_URL;
    const user = {
        email: "test2@gmail.com",
        name: "test",
        password: "1234566",
        birthday: "2023-12-29T14:57:11.873961Z"
    }
    const query = `
        mutation CreateAccount {
            createAccount(user: { email: "${user.email}", name: "${user.name}", password: "${user.password}", birthday: "${user.birthday}"  }) {
                id
                name
                active
                birthday
            }
        }
    `;
    const headers = {
        'Content-Type': 'application/json'
    };
    const response = http
        .post(
            URL,
            JSON.stringify({ query }),
            { headers }
        );

    // Creating valid user
    check(response, {
        'is status 200': (r) => r.status === 200,
        'valid response body': (r) => {
            const body = JSON.parse(r.body);
            const { id, name, active, birthday } = body.data.createAccount;
            return id && name === "test" && active && birthday;
        }
    });

    // Duplicated account
    const duplicated_account = http
        .post(
            URL,
            JSON.stringify({ query }),
            { headers }
        );
    check(duplicated_account, {
        'is status 200': (r) => r.status === 200,
        'error invalid credentials': (r) => {
            const body = JSON.parse(r.body);
            return body.errors[0].message = "Invalid Credentials";
        }
    });


    // Invalid email
    const query2 = `
        mutation CreateAccount {
            createAccount(user: { email: "invalidtest", name: "${user.name}", password: "${user.password}", birthday: "${user.birthday}"  }) {
                id
                name
                active
                birthday
            }
        }
    `;
    const invalid_email = http
        .post(
            URL,
            JSON.stringify({ query: query2 }),
            { headers }
        );
    check(invalid_email, {
        'is status 200': (r) => r.status === 200,
        'invalid email format': (r) => {
            const body = JSON.parse(r.body);
            return body.errors[0].message === "Invalid Argument: \"invalid email\"";
        }
    });
}