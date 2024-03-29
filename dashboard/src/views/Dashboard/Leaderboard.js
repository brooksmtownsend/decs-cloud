import React, { Component } from 'react';
import {
    Col,
    Row,
    Table
} from 'reactstrap';
import ScoreEntry from './ScoreEntry';


class Leaderboard extends Component {
    onUpdate = () => {
        this.setState({});
    }

    componentDidMount() {
        this.props.items.on('add', this.onUpdate);
        this.props.items.on('remove', this.onUpdate);
    }

    componentWillUnmount() {
        this.props.items.off('add', this.onUpdate);
        this.props.items.off('remove', this.onUpdate);
    }

    render() {
        // ResClient Collections are iterables, but not arrays
        return (

            <Row>
                <Col>
                    <Table responsive>
                        <thead>
                            <tr>
                                <th>Player</th>
                                <th>Score</th>
                            </tr>
                        </thead>
                        <tbody>
                            {this.props.items && Array.from(this.props.items).map(score =>
                                <ScoreEntry key={score.player} score={score} client={this.props.client} />
                            )}
                        </tbody>
                    </Table>
                </Col>
            </Row>
        );
    }
}

export default Leaderboard;
