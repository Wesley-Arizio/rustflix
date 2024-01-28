import http from "k6/http"
import { check } from 'k6';
export const options = {
    iterations: 1
};
export default function() {
    const user = {
        email: "test@gmail.com",
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
            "http://localhost:8080/graphql",
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
            "http://localhost:8080/graphql",
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
            "http://localhost:8080/graphql",
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